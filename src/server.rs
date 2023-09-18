use crate::service::Service;

pub struct Server {
    router: axum::Router<()>,
}

impl Server {
    pub fn new(paths: &[Service]) -> Server {
        let mut router = axum::Router::new();

        for path in paths {
            router = router.route(&path.path, path.into())
        }

        Server { router }
    }
    pub async fn listen(self, host: &str) {
        let srv = self.router.into_make_service();
        axum::Server::bind(&host.parse().unwrap())
            .serve(srv)
            .await
            .unwrap();
    }
}
