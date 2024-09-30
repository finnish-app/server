use crate::{
    auth::AuthSession,
    client::{
        pluggy::auth::{create_connect_token, CreateConnectTokenOutcome},
        svix::create_user_endpoint,
    },
    templates::{PluggyConnectWidgetTemplate, PluggyWidgetModalErrorTemplate},
    Env,
};

use askama_axum::IntoResponse;
use axum::http::StatusCode;

pub async fn widget(
    auth_session: AuthSession,
    env: &Env,
    pluggy_api_key: &str,
) -> impl IntoResponse {
    let Some(user) = auth_session.user else {
        return (StatusCode::UNAUTHORIZED, [("HX-Redirect", "/auth/signin")]).into_response();
    };
    tracing::debug!("User logged in");

    let Ok(webhook_url) = create_user_endpoint(&env.svix_api_key, user.id).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            PluggyWidgetModalErrorTemplate {},
        )
            .into_response();
    };

    let Ok(CreateConnectTokenOutcome::Success(connect_token)) = create_connect_token(
        pluggy_api_key,
        // "https://arst.requestcatcher.com/test", works with the damn tester
        &webhook_url, // why now with my svix webhook_url ?
        &user.email,
    )
    .await
    else {
        return (
            StatusCode::FAILED_DEPENDENCY,
            PluggyWidgetModalErrorTemplate {},
        )
            .into_response();
    };

    return PluggyConnectWidgetTemplate {
        access_token: connect_token.access_token,
    }
    .into_response();
}
