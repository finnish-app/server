use crate::{
    data_structs::{Months, MonthsIter},
    schema::{ExpenseCategory, ExpenseCategoryIter},
    util::{add_csp_to_response, generate_otp_token},
};

use askama_axum::{IntoResponse, Template};
use axum::body::Body;
use chrono::{Datelike, Month, Utc};
use strum::IntoEnumIterator;

#[derive(Template)]
#[template(path = "expenses.html")]
/// The askama template for the expenses page.
pub struct ExpensesTemplate {
    /// The current month to be displayed in English in the dropdown.
    pub current_month: Months,
    /// The expense types to be displayed in the dropdown.
    pub expense_categories: ExpenseCategoryIter,
    /// The months to be displayed in the dropdown.
    pub months: MonthsIter,
    /// The username of the logged in user.
    pub username: String,
    /// CSP nonce
    pub nonce: String,
}

impl Default for ExpensesTemplate {
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
            username: String::new(),
            nonce: String::new(),
        };
    }
}

impl ExpensesTemplate {
    pub fn into_response_with_nonce(self) -> axum::http::Response<Body> {
        let nonce = generate_otp_token();
        let nonce_str = format!("'nonce-{nonce}'");

        let mut response = Self {
            current_month: self.current_month,
            expense_categories: self.expense_categories,
            months: self.months,
            username: self.username,
            nonce,
        }
        .into_response();

        add_csp_to_response(&mut response, &nonce_str);
        return response;
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
    /// CSP nonce
    pub nonce: String,
}

impl ChangePasswordTemplate {
    pub fn into_response_with_nonce(self) -> axum::http::Response<Body> {
        let nonce = generate_otp_token();
        let nonce_str = format!("'nonce-{nonce}'");

        let mut response = Self {
            change_password: self.change_password,
            passwords_match: self.passwords_match,
            password_strength: self.password_strength,
            nonce,
        }
        .into_response();

        add_csp_to_response(&mut response, &nonce_str);
        return response;
    }
}

#[derive(Template, Default)]
#[template(path = "signin.html")]
/// The askama template for the signup page.
pub struct SignInTemplate {
    /// Message to be displayed to the user.
    pub message: String,
    /// friendly captcha secret key for getting the captcha problem
    pub frc_sitekey: String,
    /// CSP nonce
    pub nonce: String,
}

impl SignInTemplate {
    pub fn into_response_with_nonce(self) -> axum::http::Response<Body> {
        let nonce = generate_otp_token();
        let nonce_str = format!("'nonce-{nonce}'");

        let mut response = Self {
            message: self.message,
            frc_sitekey: self.frc_sitekey,
            nonce,
        }
        .into_response();

        add_csp_to_response(&mut response, &nonce_str);
        return response;
    }
}

#[derive(Template, Default)]
#[template(path = "signup.html")]
/// The askama template for the signup page.
pub struct SignUpTemplate {
    /// Friendly captcha secret key for getting the captcha problem
    pub frc_sitekey: String,
    /// CSP nonce
    pub nonce: String,
}

impl SignUpTemplate {
    pub fn into_response_with_nonce(self) -> axum::http::Response<Body> {
        let nonce = generate_otp_token();
        let nonce_str = format!("'nonce-{nonce}'");

        let mut response = Self {
            frc_sitekey: self.frc_sitekey,
            nonce,
        }
        .into_response();

        add_csp_to_response(&mut response, &nonce_str);
        return response;
    }
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
    /// CSP nonce
    pub nonce: String,
    /// Friendly captcha secret key for getting the captcha problem
    pub frc_sitekey: String,
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
    /// CSP nonce
    pub nonce: String,
}

impl MfaTemplate {
    pub fn into_response_with_nonce(self) -> axum::http::Response<Body> {
        let nonce = generate_otp_token();
        let nonce_str = format!("'nonce-{nonce}'");

        let mut response = Self {
            mfa_url: self.mfa_url,
            qr_code: self.qr_code,
            otp_auth_url: self.otp_auth_url,
            nonce,
        }
        .into_response();

        add_csp_to_response(&mut response, &nonce_str);
        return response;
    }
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
    /// CSP nonce
    pub nonce: String,
}

impl ConfirmationTemplate {
    pub fn into_response_with_nonce(self) -> axum::http::Response<Body> {
        let nonce = generate_otp_token();
        let nonce_str = format!("'nonce-{nonce}'");

        let mut response = Self {
            resend_url: self.resend_url,
            login_url: self.login_url,
            frc_sitekey: self.frc_sitekey,
            nonce,
        }
        .into_response();

        add_csp_to_response(&mut response, &nonce_str);
        return response;
    }
}
