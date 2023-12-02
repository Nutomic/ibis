use crate::database::article::DbArticle;
use crate::database::schema::{instance, instance_follow};
use crate::database::MyDataHandle;
use crate::error::{Error, MyResult};
use crate::federation::activities::follow::Follow;
use crate::federation::objects::articles_collection::DbArticleCollection;
use activitypub_federation::activity_sending::SendActivityTask;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::protocol::context::WithContext;
use activitypub_federation::traits::{ActivityHandler, Actor};
use chrono::{DateTime, Utc};
use diesel::ExpressionMethods;
use diesel::{
    insert_into, update, AsChangeset, Identifiable, Insertable, JoinOnDsl, PgConnection, QueryDsl,
    Queryable, RunQueryDsl, Selectable,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::Mutex;
use tracing::warn;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstance {
    pub id: i32,
    pub ap_id: ObjectId<DbInstance>,
    pub articles_url: CollectionId<DbArticleCollection>,
    pub inbox_url: String,
    #[serde(skip)]
    pub(crate) public_key: String,
    #[serde(skip)]
    pub(crate) private_key: Option<String>,
    #[serde(skip)]
    pub(crate) last_refreshed_at: DateTime<Utc>,
    pub local: bool,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstanceForm {
    pub ap_id: ObjectId<DbInstance>,
    pub articles_url: CollectionId<DbArticleCollection>,
    pub inbox_url: String,
    pub(crate) public_key: String,
    pub(crate) private_key: Option<String>,
    pub(crate) last_refreshed_at: DateTime<Utc>,
    pub local: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg))]
pub struct InstanceView {
    pub instance: DbInstance,
    pub followers: Vec<DbInstance>,
    pub followed: Vec<DbInstance>,
}

impl DbInstance {
    pub fn followers_url(&self) -> MyResult<Url> {
        Ok(Url::parse(&format!("{}/followers", self.ap_id.inner()))?)
    }

    pub fn follower_ids(&self, data: &Data<MyDataHandle>) -> MyResult<Vec<Url>> {
        Ok(DbInstance::read_followers(self.id, &data.db_connection)?
            .into_iter()
            .map(|f| f.ap_id.into())
            .collect())
    }

    pub async fn send_to_followers<Activity>(
        &self,
        activity: Activity,
        extra_recipients: Vec<DbInstance>,
        data: &Data<MyDataHandle>,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize + Debug + Send + Sync,
        <Activity as ActivityHandler>::Error: From<activitypub_federation::error::Error>,
        <Activity as ActivityHandler>::Error: From<Error>,
    {
        let mut inboxes: Vec<_> = DbInstance::read_followers(self.id, &data.db_connection)?
            .iter()
            .map(|f| Url::parse(&f.inbox_url).unwrap())
            .collect();
        inboxes.extend(
            extra_recipients
                .into_iter()
                .map(|i| Url::parse(&i.inbox_url).unwrap()),
        );
        self.send(activity, inboxes, data).await?;
        Ok(())
    }

    pub async fn send<Activity>(
        &self,
        activity: Activity,
        recipients: Vec<Url>,
        data: &Data<MyDataHandle>,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize + Debug + Send + Sync,
        <Activity as ActivityHandler>::Error: From<activitypub_federation::error::Error>,
    {
        let activity = WithContext::new_default(activity);
        let sends = SendActivityTask::prepare(&activity, self, recipients, data).await?;
        for send in sends {
            let send = send.sign_and_send(data).await;
            if let Err(e) = send {
                warn!("Failed to send activity {:?}: {e}", activity);
            }
        }
        Ok(())
    }

    pub fn create(form: &DbInstanceForm, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(insert_into(instance::table)
            .values(form)
            .on_conflict(instance::dsl::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: i32, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(instance::table.find(id).get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbInstance>,
        data: &Data<MyDataHandle>,
    ) -> MyResult<DbInstance> {
        let mut conn = data.db_connection.lock().unwrap();
        Ok(instance::table
            .filter(instance::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_local_instance(conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(instance::table
            .filter(instance::dsl::local.eq(true))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_local_view(conn: &Mutex<PgConnection>) -> MyResult<InstanceView> {
        let instance = DbInstance::read_local_instance(conn)?;
        let followers = DbInstance::read_followers(instance.id, conn)?;
        let followed = DbInstance::read_followed(instance.id, conn)?;

        Ok(InstanceView {
            instance,
            followers,
            followed,
        })
    }

    pub fn follow(
        follower_id_: i32,
        followed_id_: i32,
        pending_: bool,
        data: &Data<MyDataHandle>,
    ) -> MyResult<()> {
        use instance_follow::dsl::{followed_id, follower_id, pending};
        let mut conn = data.db_connection.lock().unwrap();
        insert_into(instance_follow::table)
            .values((
                follower_id.eq(follower_id_),
                followed_id.eq(followed_id_),
                pending.eq(pending_),
            ))
            .execute(conn.deref_mut())?;
        Ok(())
    }

    pub fn read_followers(id_: i32, conn: &Mutex<PgConnection>) -> MyResult<Vec<Self>> {
        use instance_follow::dsl::{followed_id, id};
        let mut conn = conn.lock().unwrap();
        Ok(instance_follow::table
            .inner_join(instance::table.on(id.eq(instance::dsl::id)))
            .filter(followed_id.eq(id_))
            .select(instance::all_columns)
            .get_results(conn.deref_mut())?)
    }

    pub fn read_followed(id_: i32, conn: &Mutex<PgConnection>) -> MyResult<Vec<Self>> {
        // TODO: is this correct?
        use instance_follow::dsl::{follower_id, id};
        let mut conn = conn.lock().unwrap();
        Ok(instance_follow::table
            .inner_join(instance::table.on(id.eq(instance::dsl::id)))
            .filter(follower_id.eq(id_))
            .select(instance::all_columns)
            .get_results(conn.deref_mut())?)
    }
}
