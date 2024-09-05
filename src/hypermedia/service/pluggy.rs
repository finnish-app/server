use crate::{
    auth::AuthSession,
    client::{
        pluggy::auth::{create_connect_token, CreateConnectTokenOutcome},
        svix::create_user_endpoint,
    },
    templates::PluggyConnectWidgetTemplate,
};

use askama_axum::IntoResponse;
use axum::http::StatusCode;
use shuttle_common::SecretStore;

pub async fn widget(
    auth_session: AuthSession,
    secret_store: &SecretStore,
    pluggy_api_key: &str,
) -> impl IntoResponse {
    let Some(user) = auth_session.user else {
        return (StatusCode::UNAUTHORIZED, [("HX-Redirect", "/auth/signin")]).into_response();
    };
    tracing::debug!("User logged in");

    let Some(svix_api_key) = secret_store.get("SVIX_API_KEY") else {
        tracing::error!("Error getting SVIX_API_KEY from secret store");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let webhook_url = create_user_endpoint(svix_api_key, "app_aarosntearnt".to_owned())
        .await
        .unwrap();

    let Ok(CreateConnectTokenOutcome::Success(connect_token)) = create_connect_token(
        pluggy_api_key,
        // "https://arst.requestcatcher.com/test", works with the damn tester
        &webhook_url, // why now with my svix webhook_url ?
        &user.email,
    )
    .await
    else {
        panic!("panik")
    };

    return PluggyConnectWidgetTemplate {
        access_token: connect_token.access_token,
    }
    .into_response();
}
