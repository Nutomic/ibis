use crate::{error::BackendResult, impls::IbisContext};
use lettre::{
    Address,
    AsyncSmtpTransport,
    AsyncTransport,
    Tokio1Executor,
    message::*,
    transport::smtp::extension::ClientId,
};
use log::{debug, warn};
use std::{str::FromStr, sync::OnceLock};
use uuid::Uuid;

pub mod notification;
pub mod reset_password;
pub mod verification;

async fn send_email(
    subject: &str,
    to_email: &str,
    html: String,
    context: &IbisContext,
) -> BackendResult<()> {
    static MAILER: OnceLock<AsyncSmtpTransport<Tokio1Executor>> = OnceLock::new();
    debug!("Sending email to {to_email} with content {html}");
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

    // use usize::MAX as the line wrap length, since lettre handles the wrapping for us
    let plain_text = html2text::from_read(html.as_bytes(), usize::MAX)?;

    let message_id = format!("<{}@{}>", Uuid::new_v4(), conf.federation.domain);
    let email = Message::builder()
        .from(email_conf.from_address.parse()?)
        .to(Mailbox::new(None, Address::from_str(to_email)?))
        .message_id(Some(message_id))
        .subject(subject)
        .multipart(MultiPart::alternative_plain_html(
            plain_text,
            html.to_string(),
        ))?;

    mailer.send(email).await?;
    Ok(())
}
