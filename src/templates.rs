use crate::{
    data_structs::{Months, MonthsIter},
    schema::{ExpenseCategory, ExpenseCategoryIter},
};

use askama_axum::Template;
use chrono::{Datelike, Month, Utc};
use strum::IntoEnumIterator;

#[derive(Template)]
#[template(path = "expenses.html")]
/// The askama template for the expenses page.
pub struct ExpensesTemplate<'a> {
    /// The current month to be displayed in English in the dropdown.
    pub current_month: Months,
    /// The expense types to be displayed in the dropdown.
    pub expense_categories: ExpenseCategoryIter,
    /// The months to be displayed in the dropdown.
    pub months: MonthsIter,
    /// The username of the logged in user.
    pub username: &'a str,
}

impl Default for ExpensesTemplate<'_> {
    fn default() -> Self {
        return Self {
            current_month: {
                Months::from_chrono_month(
                    Month::try_from(u8::try_from(Utc::now().month()).unwrap_or_else(|_| {
                        tracing::error!(
                            "Failed to convert chrono month to u8, defaulting to 1(January)"
                        );
                        return 1;
                    }))
                    .unwrap_or(Month::January),
                )
            },
            expense_categories: ExpenseCategory::iter(),
            months: Months::iter(),
            username: "",
        };
    }
}

#[derive(Template, Default)]
#[template(path = "change_password.html")]
/// The askama template for the change password page.
pub struct ChangePasswordTemplate {
    /// The url to post to change the user password.
    pub change_password: String,
    /// The url to validate that the new_password matches with its confirmation.
    pub passwords_match: String,
    /// The url to validate password strength with zxcvbn.
    pub password_strength: String,
}

#[derive(Template, Default)]
#[template(path = "auth.html")]
/// The askama template for the login page.
pub struct AuthTemplate {
    /// Triggers a message to be displayed to the user, after a failed signin or a successful signup.
    /// Or no message at all.
    pub should_print_message_in_signin: u8,
}

#[derive(Template, Default)]
#[template(path = "signin.html")]
/// The askama template for the signup page.
pub struct SignInTemplate {
    /// Message to be displayed to the user.
    pub message: String,
    /// friendly captcha secret key for getting the captcha problem
    pub frc_sitekey: String,
}

#[derive(Template, Default)]
#[template(path = "signup.html")]
/// The askama template for the signup page.
pub struct SignUpTemplate {
    /// Friendly captcha secret key for getting the captcha problem
    pub frc_sitekey: String,
}

#[derive(Template, Default)]
#[template(path = "verify.html")]
/// The askama template for the verification page.
pub struct VerificationTemplate {
    /// The url of the login page.
    pub login_url: String,
    /// The message to be displayed to the user.
    /// This is usually the error message, like user already verified, not found or invalid token.
    pub message: String,
    /// The url to GET a new verification token and send it to the user.
    pub resend_url: String,
    /// Whether to print the resend link or not, is only printed when there is an error.
    pub should_print_resend_link: bool,
}

#[derive(Template, Default)]
#[template(path = "mfa.html")]
/// The askama template for the MFA page.
pub struct MfaTemplate {
    /// The url to POST the MFA code to.
    /// If valid, the user will be logged in.
    pub mfa_url: String,
    /// QR code to be displayed to the user.
    /// It's a base64 encoded PNG.
    pub qr_code: String,
    /// The url of the qr code.
    /// So that users in mobile can click it, since scanning is not an option.
    pub otp_auth_url: String,
}

#[derive(Template, Default)]
#[template(path = "confirmation.html")]
pub struct ConfirmationTemplate {
    /// The url to GET a new confirmation code and send it to the user.
    pub resend_url: String,
    /// The url of the login page.
    pub login_url: String,
    /// friendly captcha secret key for getting the captcha problem
    pub frc_sitekey: String,
}
