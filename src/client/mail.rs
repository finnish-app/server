use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};

pub fn send_sign_up_confirmation_mail(
    to_email: &str,
    verification_code: &str,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let smtp_username = std::env::var("SMTP_USERNAME").expect("SMTP_USERNAME not set");
    let smtp_key = std::env::var("SMTP_KEY").expect("SMTP_KEY not set");
    let host = std::env::var("SMTP_HOST").expect("SMTP_HOST not set");

    let from_email = std::env::var("MAIL_FROM").expect("MAIL_FROM not set");

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
