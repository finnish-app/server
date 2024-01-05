use askama_axum::IntoResponse;
use validator::Validate;

use axum::response::Html;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
pub struct EmailInput {
    #[validate(email)]
    email: String,
}

pub async fn validate_email(input: EmailInput) -> impl IntoResponse {
    match input.validate() {
        Ok(_) => Html(format!(
            "
            <div hx-target=\"this\" hx-swap=\"outerHTML\">
                <label for=\"email\">Email</label>
                <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
                <input
                  type=\"email\"
                  name=\"email\"
                  placeholder=\"email@server.com\"
                  aria-label=\"Email\"
                  aria-invalid=\"false\"
                  autocomplete=\"email\"
                  hx-get=\"/validate/email\"
                  hx-sync=\"closest form:abort\"
                  hx-indicator=\"#ind\"
                  value=\"{}\"
                  required
                />
            </div>
            ",
            input.email
        ))
        .into_response(),
        Err(_) => Html(format!(
            "
            <div hx-target=\"this\" hx-swap=\"outerHTML\">
                <label for=\"email\">Email</label>
                <img id=\"ind\" src=\"/img/bars.svg\" class=\"htmx-indicator\"/>
                <input
                  type=\"email\"
                  name=\"email\"
                  placeholder=\"email@server.com\"
                  aria-label=\"Email\"
                  aria-invalid=\"true\"
                  autocomplete=\"email\"
                  hx-get=\"/validate/email\"
                  hx-sync=\"closest form:abort\"
                  hx-indicator=\"#ind\"
                  value=\"{}\"
                  required
                />
            </div>
            ",
            input.email
        ))
        .into_response(),
    }
}
