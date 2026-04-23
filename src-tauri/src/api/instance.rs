use serde::de::Error;
use vrchatapi::apis::{Error as ApiError, ResponseContent, instances_api::GetInstanceError};

/// A reimplementation of [vrchatapi::apis::instances_api::get_instance].
pub async fn get_instance(
    configuration: &vrchatapi::apis::configuration::Configuration,
    world_id: &str,
    instance_id: &str,
) -> Result<vrchatapi::models::Instance, ApiError<GetInstanceError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_path_world_id = world_id;
    let p_path_instance_id = instance_id;

    let uri_str = format!(
        "{}/instances/{worldId}:{instanceId}",
        configuration.base_path,
        worldId = vrchatapi::apis::urlencode(p_path_world_id),
        instanceId = p_path_instance_id,//.replace('(', "%28").replace(')', "%29"),
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream");
    let content_type = ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(ApiError::from),
            ContentType::Text => return Err(ApiError::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `vrchatapi::models::Instance`"))),
            ContentType::Unsupported(unknown_type) => return Err(ApiError::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `vrchatapi::models::Instance`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetInstanceError> = serde_json::from_str(&content).ok();
        Err(ApiError::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}


/// Internal use only
/// A content type supported by this client.
#[allow(dead_code)]
enum ContentType {
    Json,
    Text,
    Unsupported(String),
}

impl From<&str> for ContentType {
    fn from(content_type: &str) -> Self {
        if content_type.starts_with("application") && content_type.contains("json") {
            return Self::Json;
        } else if content_type.starts_with("text/plain") {
            return Self::Text;
        } else {
            return Self::Unsupported(content_type.to_string());
        }
    }
}