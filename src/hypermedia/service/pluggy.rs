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
    maybe_item_id: Option<uuid::Uuid>,
) -> impl IntoResponse {
    let Some(user) = auth_session.user else {
        return (StatusCode::UNAUTHORIZED, [("HX-Redirect", "/auth/signin")]).into_response();
    };

    // TODO: what to do with webhook stuff
    let webhook_url = match create_user_endpoint(&env.svix_api_key, user.id).await {
        Ok(url) => url,
        Err(e) => {
            tracing::error!(?e, "couldn't create user endpoint");
            // return (
            //     StatusCode::INTERNAL_SERVER_ERROR,
            //     PluggyWidgetModalErrorTemplate {},
            // )
            //     .into_response();
            "https://fina.requestcatcher.com/test".to_owned()
        }
    };

    let connect_token =
        match create_connect_token(pluggy_api_key, webhook_url, user.id, maybe_item_id).await {
            Ok(CreateConnectTokenOutcome::Success(connect_token)) => connect_token,
            Ok(
                CreateConnectTokenOutcome::Forbidden
                | CreateConnectTokenOutcome::NotFound
                | CreateConnectTokenOutcome::Internal,
            ) => {
                tracing::error!("pluggy returned error outcome on create connect token");
                return (
                    StatusCode::FAILED_DEPENDENCY,
                    PluggyWidgetModalErrorTemplate {},
                )
                    .into_response();
            }
            Err(e) => {
                tracing::error!(?e, "pluggy failed to give a response");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    PluggyWidgetModalErrorTemplate {},
                )
                    .into_response();
            }
        };

    return PluggyConnectWidgetTemplate {
        access_token: connect_token.access_token,
        item_id: maybe_item_id.map_or(String::new(), |u| u.to_string()),
    }
    .into_response();
}
