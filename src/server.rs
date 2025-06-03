use crate::service::Service;
use axum::Router;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

pub struct Server {
    router: Router,
}

impl Server {
    pub fn new(services: &[Service]) -> Server {
        let mut router = Router::new();

        for service in services {
            // Clone the service for ownership in the router
            let service_router = service.clone().to_router();
            router = router.merge(service_router);
        }

        // Add tracing middleware
        router = router.layer(TraceLayer::new_for_http());

        Server { router }
    }

    pub async fn listen(self, host: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = host.parse()?;

        info!("Starting server on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;

        axum::serve(listener, self.router).await.map_err(|e| {
            error!("Server error: {}", e);
            e.into()
        })
    }
}
