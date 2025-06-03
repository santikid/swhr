use std::{
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post, put},
    Router,
};
use tracing::{error, info};

#[derive(serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    #[default]
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
    Connect,
    Trace,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Service {
    pub path: String,
    pub script: PathBuf,
    pub dir: PathBuf,
    pub api_key: Option<String>,
    #[serde(default)]
    pub method: Method,
}

impl Service {
    pub fn to_router(&self) -> Router {
        let service = Arc::new(self.clone());

        match service.method {
            Method::Get => Router::new()
                .route(&service.path, get(handle_webhook))
                .with_state(service),
            Method::Post => Router::new()
                .route(&service.path, post(handle_webhook))
                .with_state(service),
            Method::Put => Router::new()
                .route(&service.path, put(handle_webhook))
                .with_state(service),
            Method::Delete => Router::new()
                .route(&service.path, delete(handle_webhook))
                .with_state(service),
            Method::Patch => Router::new()
                .route(&service.path, patch(handle_webhook))
                .with_state(service),
            _ => {
                error!(
                    "Method {:?} not supported for path {}",
                    service.method, service.path
                );
                Router::new()
            }
        }
    }
}

async fn handle_webhook(
    headers: HeaderMap,
    State(service): State<Arc<Service>>,
    body: String,
) -> (StatusCode, String) {
    // Check if API key is needed and correct
    if let Some(want) = &service.api_key {
        let api_key = headers.get("x-api-key").and_then(|v| v.to_str().ok());

        match api_key {
            Some(provided) if provided == want => {
                // API key is valid, continue
            }
            Some(_) => {
                return (StatusCode::UNAUTHORIZED, "Invalid API key".to_string());
            }
            None => {
                return (StatusCode::UNAUTHORIZED, "Missing API key".to_string());
            }
        }
    }

    // Execute the webhook script
    match Command::new(&service.script)
        .env("WEBHOOK_BODY", body.clone())
        .current_dir(&service.dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => {
            info!("Started {} (PID {})", service.script.display(), child.id());
        }
        Err(e) => {
            error!("Failed to start {}: {}", service.script.display(), e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to execute webhook: {e}"),
            );
        }
    }

    (StatusCode::OK, "Webhook executed successfully".to_string())
}
