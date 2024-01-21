use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use shuttle_secrets::SecretStore;

pub fn send_sign_up_confirmation_mail(
    secret_store: &SecretStore,
    to_email: &str,
    verification_code: &str,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let smtp_username = secret_store.get("SMTP_USERNAME").unwrap_or_else(|| {
        tracing::warn!("SMTP_USERNAME not set, using default");
        "".to_string()
    });
    let smtp_key = secret_store.get("SMTP_KEY").unwrap_or_else(|| {
        tracing::warn!("SMTP_KEY not set, using default");
        "".to_string()
    });
    let host = secret_store.get("SMTP_HOST").unwrap_or_else(|| {
        tracing::warn!("SMTP_HOST not set, using default");
        "".to_string()
    });

    let from_email = secret_store.get("MAIL_FROM").unwrap_or_else(|| {
        tracing::warn!("MAIL_FROM not set, using default");
        "".to_string()
    });

    let email: Message = Message::builder()
        .from(from_email.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject("Confirm your email")
        .header(ContentType::TEXT_HTML)
        .body(format!("
            <html>
                <body>
                    <h1>Hi there!</h1>
                    <p>Thanks for signing up for finnish!</p>
                    <p>Click <a href=\"https://finnish.shuttleapp.rs/auth/verify-email/{}\">here</a> to verify your email.</p>
                </body>
            </html>",
            verification_code
        )).unwrap();

    let creds = Credentials::new(smtp_username, smtp_key);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(&host)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)
}
