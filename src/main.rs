use anyhow::Result;
use clap::Parser;
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod server;
mod service;

/// Simple Webhook Runner (swhr)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "swhr.yaml")]
    config: String,

    /// Listen address in format IP:PORT
    #[arg(short, long, default_value = "127.0.0.1:3344")]
    listen: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(serde::Serialize, Clone, serde::Deserialize, Debug)]
struct Config {
    services: Vec<service::Service>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize the tracing subscriber
    setup_logging(&args.log_level)?;

    info!("Starting Simple Webhook Runner (swhr)");
    info!("Loading configuration from {}", args.config);

    let cfg = match load_config(&args.config) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load config: {}", err);
            return Err(err);
        }
    };

    info!("Loaded {} service(s)", cfg.services.len());

    for (i, service) in cfg.services.iter().enumerate() {
        info!(
            "Service #{}: {} -> {} (method: {:?})",
            i + 1,
            service.path,
            service.script.display(),
            service.method
        );
    }

    // Create and start the server
    let srv = server::Server::new(&cfg.services);
    match srv.listen(&args.listen).await {
        Ok(_) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(err) => {
            error!("Server error: {}", err);
            Err(anyhow::anyhow!("Server error: {}", err))
        }
    }
}

fn load_config(path: &str) -> Result<Config> {
    // Load and parse configuration
    let cfg = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            error!("Failed to read config file {}: {}", path, err);
            return Err(anyhow::anyhow!("Failed to read config file: {}", err));
        }
    };

    match serde_yaml::from_str(&cfg) {
        Ok(parsed) => Ok(parsed),
        Err(err) => {
            error!("Failed to parse config: {}", err);
            Err(anyhow::anyhow!("Failed to parse config: {}", err))
        }
    }
}

fn setup_logging(log_level: &str) -> Result<()> {
    let filter = log_level
        .to_uppercase()
        .parse::<LevelFilter>()
        .unwrap_or(LevelFilter::INFO);
    info!("Setting log level to {}", filter);

    let env_filter = EnvFilter::builder()
        .with_default_directive(filter.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(env_filter)
        .try_init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;

    Ok(())
}
