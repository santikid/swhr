use std::{path::PathBuf, process::Command};

use axum::{
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post, put, MethodRouter},
};

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Service {
    pub path: String,
    pub script: PathBuf,
    pub dir: PathBuf,
    pub api_key: Option<String>,
    #[serde(default)]
    pub method: Method,
}

impl From<&Service> for MethodRouter {
    fn from(value: &Service) -> Self {
        let value = value.clone();
        let handler = {
            async move |headers: HeaderMap, body: String| {
                // check if api key is needed and correct
                if let Some(want) = value.api_key {
                    if let Some(provided_raw) = headers.get("x-api-key") && let Ok(provided) = provided_raw.to_str() {
                            if provided != want {
                                return (StatusCode::UNAUTHORIZED, "invalid api key".to_string());
                            }
                        } else {
                            return (StatusCode::UNAUTHORIZED, "missing api key".to_string());
                        }
                }

                match Command::new(&value.script)
                    .env("WEBHOOK_BODY", body)
                    .current_dir(&value.dir)
                    .spawn()
                {
                    Ok(e) => {
                        println!(
                            "{} (PID {}) started",
                            &value.script.to_string_lossy(),
                            e.id()
                        );
                    }
                    Err(e) => {
                        println!("{} failed to start: {}", &value.script.to_string_lossy(), e)
                    }
                }
                (StatusCode::OK, "ok".to_string())
            }
        };
        match value.method {
            Method::Get => get(handler),
            Method::Post => post(handler),
            Method::Put => put(handler),
            Method::Patch => patch(handler),
            Method::Delete => delete(handler),
            _ => panic!("method not supported"),
        }
    }
}
