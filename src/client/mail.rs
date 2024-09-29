use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

pub struct EmailSecrets<'a> {
    pub smtp_username: &'a str,
    pub smtp_host: &'a str,
    pub smtp_key: &'a str,
    pub mail_from: &'a str,
}

// TODO: create a mailer object and add it to application state
// instead of creating on every call
pub fn send_email(
    email_secrets: &EmailSecrets,
    to_email: &str,
    subject: &str,
    body: &str,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let email: Message = Message::builder()
        .from(email_secrets.mail_from.parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(body.to_owned())
        .unwrap();

    let creds = Credentials::new(
        email_secrets.smtp_username.to_owned(),
        email_secrets.smtp_key.to_owned(),
    );

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay(email_secrets.smtp_host)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    mailer.send(&email)
}

pub fn send_sign_up_confirmation_mail(
    email_secrets: &EmailSecrets,
    to_email: &str,
    verification_code: &str,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let body = format!(
        "<html>
            <body>
                <h1>Hi there!</h1>
                <p>Thanks for signing up for finnish!</p>
                <p>Click <a href=\"https://finnish.shuttleapp.rs/auth/verify-email/{verification_code}\">here</a> to verify your email.</p>
            </body>
        </html>"
    );

    send_email(
        email_secrets,
        to_email,
        "Welcome to finnish! Confirm your email.",
        &body,
    )
}

pub fn send_forgot_password_mail(
    email_secrets: &EmailSecrets,
    to_email: &str,
    verification_code: &str,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let body = format!(
        "<html>
            <body>
                <h1>Hi there!</h1>
                <p>Having trouble login in or remembering your password?</p>
                <p>Click <a href=\"https://finnish.shuttleapp.rs/auth/reset-password/{verification_code}\">here</a> to reset your password.</p>
            </body>
        </html>"
    );

    send_email(email_secrets, to_email, "Reset your password", &body)
}
