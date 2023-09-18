#![feature(async_closure)]
#![feature(let_chains)]

use clap::Parser;

mod server;
mod service;

/// Simple Webhook Server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to config
    #[arg(short, long, default_value = "swhr.yaml")]
    config: String,

    /// listen address
    #[arg(short, long, default_value = "127.0.0.1:3344")]
    listen: String,
}

#[derive(serde::Serialize, Clone, serde::Deserialize)]
struct Config {
    services: Vec<service::Service>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let cfg = std::fs::read_to_string(args.config).expect("config not found");
    let cfg: Config = serde_yaml::from_str(&cfg).expect("failed to parse config");

    let srv = server::Server::new(&cfg.services);
    srv.listen(&args.listen).await;
}
