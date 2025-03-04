use crate::{
    common::{newtypes::LocalUserId, user::LocalUserView},
    error::BackendResult,
    impls::IbisContext,
    schema::{email_verification, local_user},
};
use chrono::{DateTime, Utc};
use diesel::{
    dsl::{IntervalDsl, now},
    sql_types::Timestamptz,
    *,
};
use lettre::{
    Address,
    AsyncSmtpTransport,
    AsyncTransport,
    Tokio1Executor,
    message::*,
    transport::smtp::extension::ClientId,
};
use log::warn;
use std::{ops::DerefMut, str::FromStr, sync::OnceLock};
use uuid::Uuid;

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

pub async fn send_validation_email(
    to_user: &LocalUserView,
    new_email: &str,
    context: &IbisContext,
) -> BackendResult<()> {
    let mut conn = context.db_pool.get()?;
    let domain = &context.conf.federation.domain;
    let form = EmailVerificationForm {
        local_user_id: to_user.local_user.id,
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
        r#"Please click the link below to verify your email address for the account @{}@{}. Ignore this email if the account isn't yours.<br><br>, <a href=\"{verify_link}\">Verify your email</a>"#,
        to_user.person.username, domain
    );

    send_email("Registration for Ibis", to_user, body, context).await?;
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

pub async fn send_email(
    subject: &str,
    to_user: &LocalUserView,
    html: String,
    context: &IbisContext,
) -> BackendResult<()> {
    static MAILER: OnceLock<AsyncSmtpTransport<Tokio1Executor>> = OnceLock::new();
    let conf = &context.conf;
    let Some(email_conf) = conf.email.clone() else {
        warn!("Email not configured");
        return Ok(());
    };

    let mailer = MAILER.get_or_init(|| {
        AsyncSmtpTransport::<Tokio1Executor>::from_url(&email_conf.connection_url)
            .expect("init email transport")
            .hello_name(ClientId::Domain(conf.federation.domain.clone()))
            .build()
    });

    let Some(to_email) = &to_user.local_user.email else {
        return Ok(());
    };
    let to_name = to_user
        .person
        .display_name
        .as_ref()
        .unwrap_or(&to_user.person.username)
        .to_string();

    // use usize::MAX as the line wrap length, since lettre handles the wrapping for us
    let plain_text = html2text::from_read(html.as_bytes(), usize::MAX)?;

    let message_id = format!("<{}@{}>", Uuid::new_v4(), conf.federation.domain);
    let email = Message::builder()
        .from(email_conf.from_address.parse()?)
        .to(Mailbox::new(Some(to_name), Address::from_str(to_email)?))
        .message_id(Some(message_id))
        .subject(subject)
        .multipart(MultiPart::alternative_plain_html(
            plain_text,
            html.to_string(),
        ))?;

    mailer.send(email).await?;

    Ok(())
}
