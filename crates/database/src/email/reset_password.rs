use super::send_email;
use crate::{
    common::{newtypes::LocalUserId, user::LocalUserView},
    error::BackendResult,
    impls::{IbisContext, user::LocalUserViewQuery},
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use diesel::{
    dsl::{IntervalDsl, now},
    sql_types::Timestamptz,
    *,
};
use ibis_database_schema::password_reset_request;
use uuid::Uuid;

#[derive(PartialEq, Eq, Debug, Queryable, Selectable, Identifiable)]
#[ diesel(table_name = password_reset_request)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PasswordResetRequest {
    id: i32,
    pub local_user_id: LocalUserId,
    token: String,
    published: DateTime<Utc>,
}

#[derive(Insertable, AsChangeset)]
#[ diesel(table_name = password_reset_request)]
struct PasswordResetRequestForm {
    local_user_id: LocalUserId,
    token: String,
}

impl PasswordResetRequest {
    pub async fn create(email: &str, context: &IbisContext) -> BackendResult<()> {
        let local_user_view = LocalUserView::read(LocalUserViewQuery::Email(email), context)?;
        let Some(email) = local_user_view.local_user.email else {
            return Err(anyhow!("user has no email address").into());
        };

        let mut conn = context.db_pool.get()?;
        let form = PasswordResetRequestForm {
            local_user_id: local_user_view.local_user.id,
            token: Uuid::new_v4().to_string(),
        };
        let token = insert_into(password_reset_request::table)
            .values(form)
            .get_result::<PasswordResetRequest>(&mut conn)?;

        let domain = &context.conf.federation.domain;
        let reset_link = format!("{}/account/reset_password?token={}", domain, &token.token);
        let body = format!(
            r#"<h1>Password Reset Request for {}</h1><br><a href=\"{reset_link}\">Click here to reset your password</a>"#,
            local_user_view.person.username
        );
        send_email("Password reset", &email, body, context).await?;

        Ok(())
    }

    pub fn read_and_delete(token_: &str, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(delete(password_reset_request::table)
            .filter(password_reset_request::token.eq(token_))
            .filter(password_reset_request::published.gt(now.into_sql::<Timestamptz>() - 1.days()))
            .get_result(&mut conn)?)
    }
}
