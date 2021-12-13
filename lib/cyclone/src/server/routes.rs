use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use axum::{handler::get, routing::BoxRoute, AddExtensionLayer, Router};
use telemetry::prelude::*;
use tokio::sync::mpsc;

use super::{handlers, server::ShutdownSource, tower::LimitRequestLayer, Config};
use crate::server::watch;

pub struct State {
    lang_server_path: PathBuf,
}

impl State {
    /// Gets a reference to the state's lang server path.
    pub fn lang_server_path(&self) -> &Path {
        &self.lang_server_path
    }
}

impl From<&Config> for State {
    fn from(value: &Config) -> Self {
        Self {
            lang_server_path: value.lang_server_path().to_path_buf(),
        }
    }
}

pub struct WatchKeepalive {
    tx: mpsc::Sender<()>,
    timeout: Duration,
}

impl WatchKeepalive {
    pub fn clone_tx(&self) -> mpsc::Sender<()> {
        self.tx.clone()
    }

    /// Gets a reference to the watch keepalive tx's timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[must_use]
pub fn routes(config: &Config, shutdown_tx: mpsc::Sender<ShutdownSource>) -> Router<BoxRoute> {
    let shared_state = Arc::new(State::from(config));

    let mut router = Router::new()
        .route(
            "/liveness",
            get(handlers::liveness).head(handlers::liveness),
        )
        .route(
            "/readiness",
            get(handlers::readiness).head(handlers::readiness),
        )
        .nest("/execute", execute_routes(config, shutdown_tx.clone()))
        .boxed();

    if let Some(watch_timeout) = config.watch() {
        debug!("enabling watch endpoint");
        let (keepalive_tx, keepalive_rx) = mpsc::channel::<()>(4);

        tokio::spawn(watch::watch_timeout_task(
            watch_timeout,
            shutdown_tx,
            keepalive_rx,
        ));

        let watch_keepalive = WatchKeepalive {
            tx: keepalive_tx,
            timeout: watch_timeout,
        };

        router = router
            .or(Router::new()
                .route("/watch", get(handlers::ws_watch))
                .layer(AddExtensionLayer::new(Arc::new(watch_keepalive))))
            .boxed();
    }

    router.layer(AddExtensionLayer::new(shared_state)).boxed()
}

fn execute_routes(config: &Config, shutdown_tx: mpsc::Sender<ShutdownSource>) -> Router<BoxRoute> {
    let mut router = Router::new().boxed();

    if config.enable_ping() {
        debug!("enabling ping endpoint");
        router = router
            .or(Router::new().route("/ping", get(handlers::ws_execute_ping)))
            .boxed();
    }
    if config.enable_resolver() {
        debug!("enabling resolver endpoint");
        router = router
            .or(Router::new().route("/resolver", get(handlers::ws_execute_resolver)))
            .boxed();
    }

    router
        .layer(LimitRequestLayer::new(config.limit_requests(), shutdown_tx))
        // TODO(fnichol): we are going to need this, mark my words...
        // .handle_error(convert_tower_error_into_reponse)
        .boxed()
}

// TODO(fnichol): we are going to need this, mark my words...
//
//
// fn convert_tower_error_into_reponse(err: BoxError) -> Result<Response<Full<Bytes>>, Infallible> {
//     // TODO(fnichol): more to do here, see:
//     // https://github.com/bwalter/rust-axum-scylla/blob/main/src/routing/mod.rs
//     Ok((
//         StatusCode::INTERNAL_SERVER_ERROR,
//         Json(json!({ "error": err.to_string() })),
//     )
//         .into_response())
// }