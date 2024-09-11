use reqwest::Url;
use svix::api::{ApplicationIn, ApplicationOut, EndpointIn, Svix};

const ENDPOINT_URL_PREFIX: &str = "https://finnish.shuttleapp.rs/webhooks";

pub async fn create_user_app(svix_api_key: &str, user_id: i32) -> anyhow::Result<ApplicationOut> {
    let svix = Svix::new(svix_api_key.to_owned(), None);
    let app = svix
        .application()
        .create(
            ApplicationIn {
                name: format!("finnish-{user_id}"),
                uid: Some(user_id.to_string()),
                ..ApplicationIn::default()
            },
            None,
        )
        .await?;

    Ok(app)
}

pub async fn create_user_endpoint(svix_api_key: &str, user_id: i32) -> anyhow::Result<String> {
    let svix = Svix::new(svix_api_key.to_owned(), None);

    let base = Url::parse(ENDPOINT_URL_PREFIX)?;
    let uuid = uuid::Uuid::new_v4().to_string();
    let joined = base.join(&uuid)?;

    let endpoint = svix
        .endpoint()
        .create(
            user_id.to_string(),
            EndpointIn {
                url: joined.to_string(),
                description: Some("Pluggy connect endpoint".to_string()),
                ..EndpointIn::default()
            },
            None,
        )
        .await?;

    Ok(endpoint.url)
}
