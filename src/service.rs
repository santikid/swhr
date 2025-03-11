use std::{path::PathBuf, process::Command, sync::Arc};

use axum::{
    extract::{State, Json},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Router,
    body::Bytes,
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
    pub fn to_router(self) -> Router {
        let service = Arc::new(self);
        
        match service.method {
            Method::Get => Router::new().route(&service.path, get(|h, s, b| handle_webhook(h, s, b))).with_state(service),
            Method::Post => Router::new().route(&service.path, post(|h, s, b| handle_webhook(h, s, b))).with_state(service),
            Method::Put => Router::new().route(&service.path, put(|h, s, b| handle_webhook(h, s, b))).with_state(service),
            Method::Delete => Router::new().route(&service.path, delete(|h, s, b| handle_webhook(h, s, b))).with_state(service),
            Method::Patch => Router::new().route(&service.path, patch(|h, s, b| handle_webhook(h, s, b))).with_state(service),
            _ => {
                error!("Method {:?} not supported for path {}", service.method, service.path);
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
        let api_key = headers.get("x-api-key")
            .and_then(|v| v.to_str().ok());
            
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
        .spawn()
    {
        Ok(child) => {
            info!(
                "Started {} (PID {})",
                service.script.display(),
                child.id()
            );
        }
        Err(e) => {
            error!("Failed to start {}: {}", service.script.display(), e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR, 
                format!("Failed to execute webhook: {}", e)
            );
        }
    }

    (StatusCode::OK, "Webhook executed successfully".to_string())
}
