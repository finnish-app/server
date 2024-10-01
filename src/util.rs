use axum::{body::Body, http::Response};
use axum_helmet::ContentSecurityPolicy;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

/// Generates a random string of 128 characters to be used as an email verification token.
pub fn generate_verification_token() -> String {
    return thread_rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect();
}

/// Generates a random string of 20 characters to be used as a one time password seed.
pub fn generate_otp_token() -> String {
    return thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20) // recommended length of secret is 160 bits: https://www.rfc-editor.org/rfc/rfc4226#section-4
        .map(char::from)
        .collect();
}

/// Receives something that implements `IntoResponse` and adds a CSP header to it.
pub fn add_csp_to_response(response: &mut Response<Body>, nonce_str: &str) {
    if let Ok(csp) = ContentSecurityPolicy::new()
        .default_src(vec!["'self'", "https://api.friendlycaptcha.com"])
        .base_uri(vec!["'none'"])
        .font_src(vec!["'none'"])
        .form_action(vec!["'none'"])
        //.frame_src(vec!["'none'"])
        .frame_src(vec!["https://connect.pluggy.ai"])
        .frame_ancestors(vec!["'none'"])
        .img_src(vec!["'self'", "data:"])
        .object_src(vec!["'none'"])
        .script_src(vec!["'wasm-unsafe-eval'", "'strict-dynamic'", nonce_str])
        .style_src(vec!["'self'", "https:", "'unsafe-inline'"])
        .worker_src(vec!["blob:"])
        .upgrade_insecure_requests()
        .to_string()
        .parse()
    {
        response
            .headers_mut()
            .insert("content-security-policy", csp);
    }
}
