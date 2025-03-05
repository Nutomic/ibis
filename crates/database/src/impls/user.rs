use crate::{
    DbUrl,
    common::{
        instance::InstanceFollow,
        newtypes::{LocalUserId, PersonId},
        user::{LocalUser, LocalUserView, Person},
        utils::http_protocol_str,
    },
    error::BackendResult,
    impls::{IbisContext, coalesce, lower},
    schema::{instance, instance_follow, local_user, oauth_account, person},
    utils::generate_keypair,
};
use anyhow::anyhow;
use bcrypt::{DEFAULT_COST, hash};
use chrono::{DateTime, Utc};
use diesel::{
    AsChangeset,
    BoolExpressionMethods,
    ExpressionMethods,
    Insertable,
    JoinOnDsl,
    PgTextExpressionMethods,
    QueryDsl,
    Queryable,
    RunQueryDsl,
    Selectable,
    dsl::not,
    insert_into,
};
use std::ops::DerefMut;
use url::Url;

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = local_user, check_for_backend(diesel::pg::Pg))]
pub struct LocalUserInsertForm {
    pub password_encrypted: Option<String>,
    pub person_id: PersonId,
    pub admin: bool,
    pub email: Option<String>,
    pub email_verified: bool,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = local_user, check_for_backend(diesel::pg::Pg))]
pub struct LocalUserUpdateForm {
    pub email_notifications: Option<bool>,
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

#[derive(Clone, PartialEq, Eq, Debug, Queryable, Selectable)]
#[ diesel(table_name = oauth_account)]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// An auth account method.
pub struct OAuthAccount {
    pub local_user_id: LocalUserId,
    pub oauth_issuer_url: DbUrl,
    pub oauth_user_id: String,
    pub published: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[ diesel(table_name = oauth_account)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OAuthAccountInsertForm {
    pub local_user_id: LocalUserId,
    pub oauth_issuer_url: DbUrl,
    pub oauth_user_id: String,
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

    pub fn update(
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
            let domain = &context.conf.federation.domain;
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

#[derive(Debug)]
pub enum LocalUserViewQuery<'a> {
    LocalNameOrEmail(&'a str),
    Oauth(DbUrl, &'a str),
    Email(&'a str),
}

impl LocalUserView {
    pub fn create(
        username: String,
        password: Option<String>,
        admin: bool,
        email: Option<String>,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        let domain = &context.conf.federation.domain;
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

        let local_user_form = LocalUserInsertForm {
            password_encrypted: password.map(|p| hash(p, DEFAULT_COST)).transpose()?,
            person_id: person.id,
            admin,
            email,
            email_verified: false,
        };

        let local_user = insert_into(local_user::table)
            .values(local_user_form)
            .get_result::<LocalUser>(conn.deref_mut())?;

        Ok(Self { local_user, person })
    }

    pub fn read(params: LocalUserViewQuery, context: &IbisContext) -> BackendResult<LocalUserView> {
        use LocalUserViewQuery::*;
        let mut conn = context.db_pool.get()?;
        let mut query = local_user::table
            .inner_join(person::table)
            .left_join(oauth_account::table)
            .select((person::all_columns, local_user::all_columns))
            .into_boxed();
        query = match params {
            LocalNameOrEmail(name_or_email) => query.filter(person::local).filter(
                person::username
                    .eq(name_or_email)
                    .or(local_user::email.eq(name_or_email)),
            ),
            Oauth(issuer, user_id) => query
                .filter(oauth_account::oauth_issuer_url.eq(issuer))
                .filter(oauth_account::oauth_user_id.eq(user_id)),
            Email(email) => query.filter(local_user::email.eq(email)),
        };
        Ok(query.get_result::<LocalUserView>(conn.deref_mut())?)
    }
}

impl LocalUser {
    pub fn check_username_taken(username: &str, context: &IbisContext) -> BackendResult<()> {
        use diesel::dsl::{exists, select};
        let mut conn = context.db_pool.get()?;
        select(not(exists(
            person::table
                .filter(person::local)
                .filter(lower(person::username).eq(username.to_lowercase())),
        )))
        .get_result::<bool>(conn.deref_mut())?
        .then_some(())
        .ok_or(anyhow!("Username already exists").into())
    }

    pub fn check_email_taken(email: &str, context: &IbisContext) -> BackendResult<()> {
        use diesel::dsl::{exists, select};
        let mut conn = context.db_pool.get()?;
        select(not(exists(local_user::table.filter(
            lower(coalesce(local_user::email, "")).eq(email.to_lowercase()),
        ))))
        .get_result::<bool>(conn.deref_mut())?
        .then_some(())
        .ok_or(anyhow!("Email is taken").into())
    }

    pub fn update_password(
        password: String,
        id: LocalUserId,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        diesel::update(local_user::table.find(id))
            .set(local_user::password_encrypted.eq(hash(password, DEFAULT_COST)?))
            .execute(conn.deref_mut())?;
        Ok(())
    }

    pub fn update(
        form: &LocalUserUpdateForm,
        id: LocalUserId,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        diesel::update(local_user::table.find(id))
            .set(form)
            .execute(conn.deref_mut())?;
        Ok(())
    }
}

impl OAuthAccount {
    pub fn create(form: &OAuthAccountInsertForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(oauth_account::table)
            .values(form)
            .get_result::<Self>(conn.deref_mut())?)
    }
}
