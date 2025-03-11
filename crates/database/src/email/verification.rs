use super::send_email;
use crate::{
    common::{newtypes::LocalUserId, user::LocalUser},
    error::BackendResult,
    impls::IbisContext,
};
use chrono::{DateTime, Utc};
use diesel::{
    dsl::{IntervalDsl, now},
    sql_types::Timestamptz,
    *,
};
use ibis_database_schema::{email_verification, local_user};
use std::ops::DerefMut;

#[derive(Clone, Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = email_verification)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EmailVerification {
    pub id: i32,
    pub local_user_id: LocalUserId,
    pub email: String,
    pub verification_token: String,
    pub published: DateTime<Utc>,
}

#[derive(Insertable, AsChangeset)]
#[ diesel(table_name = email_verification)]
pub struct EmailVerificationForm {
    pub local_user_id: LocalUserId,
    pub email: String,
    pub verification_token: String,
}

pub async fn send_verification_email(
    to_user: &LocalUser,
    new_email: &str,
    context: &IbisContext,
) -> BackendResult<()> {
    let mut conn = context.db_pool.get()?;
    let domain = &context.conf.federation.domain;
    let form = EmailVerificationForm {
        local_user_id: to_user.id,
        email: new_email.to_string(),
        verification_token: uuid::Uuid::new_v4().to_string(),
    };
    let verify_link = format!(
        "{}/account/verify_email?token={}",
        domain, &form.verification_token
    );
    insert_into(email_verification::table)
        .values(form)
        .execute(conn.deref_mut())?;

    let body = format!(
        r#"Please click the link below to verify your email address for the account on {}. Ignore this email if the account isn't yours.<br><br>
        <a href="{verify_link}">Verify your email</a>"#,
        domain
    );

    send_email("Registration for Ibis", new_email, body, context).await?;
    Ok(())
}

pub fn set_email_verified(token: &str, context: &IbisContext) -> BackendResult<()> {
    let mut conn = context.db_pool.get()?;
    // read the token, dont delete it yet because this may be called multiple times from ssr/csr
    let verification: EmailVerification = email_verification::table
        .filter(email_verification::verification_token.eq(token))
        .get_result(conn.deref_mut())?;

    // mark email as validated
    update(local_user::table.filter(local_user::id.eq(verification.local_user_id)))
        .set((
            local_user::email.eq(verification.email),
            local_user::email_verified.eq(true),
        ))
        .execute(conn.deref_mut())?;

    // delete old tokens
    delete(
        email_verification::table
            .filter(email_verification::published.lt(now.into_sql::<Timestamptz>() - 7.days())),
    )
    .execute(conn.deref_mut())?;
    Ok(())
}
