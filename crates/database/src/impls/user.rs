use crate::{
    DbUrl,
    common::{
        instance::InstanceFollow,
        newtypes::PersonId,
        user::{LocalUser, LocalUserView, Person},
        utils::http_protocol_str,
    },
    error::BackendResult,
    impls::IbisContext,
    schema::{instance, instance_follow, local_user, person},
    utils::generate_keypair,
};
use bcrypt::{DEFAULT_COST, hash};
use chrono::{DateTime, Utc};
use diesel::{
    AsChangeset,
    ExpressionMethods,
    Insertable,
    JoinOnDsl,
    PgTextExpressionMethods,
    QueryDsl,
    RunQueryDsl,
    insert_into,
};
use std::ops::DerefMut;
use url::Url;

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = local_user, check_for_backend(diesel::pg::Pg))]
pub struct LocalUserForm {
    pub password_encrypted: String,
    pub person_id: PersonId,
    pub admin: bool,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = person, check_for_backend(diesel::pg::Pg))]
pub struct PersonInsertForm {
    pub username: String,
    pub ap_id: DbUrl,
    pub inbox_url: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = person, check_for_backend(diesel::pg::Pg))]
pub struct PersonUpdateForm {
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

impl Person {
    pub fn create(person_form: &PersonInsertForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(person::table)
            .values(person_form)
            .on_conflict(person::dsl::ap_id)
            .do_update()
            .set(person_form)
            .get_result::<Person>(conn.deref_mut())?)
    }

    pub fn read(id: PersonId, context: &IbisContext) -> BackendResult<Person> {
        let mut conn = context.db_pool.get()?;
        Ok(person::table.find(id).get_result(conn.deref_mut())?)
    }

    pub fn read_admin(context: &IbisContext) -> BackendResult<Person> {
        let mut conn = context.db_pool.get()?;
        Ok(person::table
            .inner_join(local_user::table)
            .filter(local_user::admin)
            .select(person::all_columns)
            .get_result(conn.deref_mut())?)
    }

    pub fn create_local(
        username: String,
        password: String,
        admin: bool,
        context: &IbisContext,
    ) -> BackendResult<LocalUserView> {
        let mut conn = context.db_pool.get()?;
        let domain = &context.config.federation.domain;
        let ap_id = Url::parse(&format!(
            "{}://{domain}/user/{username}",
            http_protocol_str()
        ))?
        .into();
        let inbox_url = format!("{}://{domain}/inbox", http_protocol_str());
        let keypair = generate_keypair()?;
        let person_form = PersonInsertForm {
            username,
            ap_id,
            inbox_url,
            public_key: keypair.public_key,
            private_key: Some(keypair.private_key),
            last_refreshed_at: Utc::now(),
            local: true,
            display_name: None,
            bio: None,
        };

        let person = insert_into(person::table)
            .values(person_form)
            .get_result::<Person>(conn.deref_mut())?;

        let local_user_form = LocalUserForm {
            password_encrypted: hash(password, DEFAULT_COST)?,
            person_id: person.id,
            admin,
        };

        let local_user = insert_into(local_user::table)
            .values(local_user_form)
            .get_result::<LocalUser>(conn.deref_mut())?;

        Ok(LocalUserView { local_user, person })
    }

    pub fn read_from_ap_id(ap_id: &DbUrl, context: &IbisContext) -> BackendResult<Person> {
        let mut conn = context.db_pool.get()?;
        Ok(person::table
            .filter(person::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_from_name(
        username: &str,
        domain: &Option<String>,
        context: &IbisContext,
    ) -> BackendResult<Person> {
        let mut conn = context.db_pool.get()?;
        let mut query = person::table
            .filter(person::username.eq(username))
            .select(person::all_columns)
            .into_boxed();
        query = if let Some(domain) = domain {
            let domain_pattern = format!("%://{domain}/%");
            query
                .filter(person::ap_id.ilike(domain_pattern))
                .filter(person::local.eq(false))
        } else {
            query.filter(person::local.eq(true))
        };
        Ok(query.get_result(conn.deref_mut())?)
    }

    pub fn update_profile(
        form: &PersonUpdateForm,
        id: PersonId,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        diesel::update(person::table.find(id))
            .set(form)
            .execute(conn.deref_mut())?;
        Ok(())
    }

    pub fn read_local_from_name(
        username: &str,
        context: &IbisContext,
    ) -> BackendResult<LocalUserView> {
        let mut conn = context.db_pool.get()?;
        let (person, local_user) = person::table
            .inner_join(local_user::table)
            .filter(person::dsl::local)
            .filter(person::dsl::username.eq(username))
            .get_result::<(Person, LocalUser)>(conn.deref_mut())?;
        Ok(LocalUserView { person, local_user })
    }

    pub fn read_following(
        id_: PersonId,
        context: &IbisContext,
    ) -> BackendResult<Vec<InstanceFollow>> {
        use instance_follow::dsl::{follower_id, instance_id};
        let mut conn = context.db_pool.get()?;
        Ok(instance_follow::table
            .inner_join(instance::table.on(instance_id.eq(instance::dsl::id)))
            .filter(follower_id.eq(id_))
            .select((instance::all_columns, instance_follow::pending))
            .get_results(conn.deref_mut())?)
    }

    /// Ghost user serves as placeholder for deleted accounts
    pub fn ghost(context: &IbisContext) -> BackendResult<Person> {
        let username = "ghost";
        let read = Person::read_from_name(username, &None, context);
        if read.is_ok() {
            read
        } else {
            let domain = &context.config.federation.domain;
            let ap_id = Url::parse(&format!(
                "{}://{domain}/user/{username}",
                http_protocol_str()
            ))?
            .into();
            let inbox_url = format!("{}://{domain}/inbox", http_protocol_str());
            let keypair = generate_keypair()?;
            let person_form = PersonInsertForm {
                username: username.to_string(),
                ap_id,
                inbox_url,
                public_key: keypair.public_key,
                private_key: Some(keypair.private_key),
                last_refreshed_at: Utc::now(),
                local: true,
                display_name: None,
                bio: None,
            };
            Person::create(&person_form, context)
        }
    }
}
