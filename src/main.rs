mod api;
mod db;
mod handoff;
mod logger;
mod postgres;

use std::{sync::Arc, time::Instant};

use axum::middleware;
use tokio::sync::{Mutex, RwLock};

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());

    // Take the database over from the old revision during the restore, so no changes are lost and only one instance archives to GCS.
    let restored = postgres::start(handoff::stop_serving_revision()).await;
    let server = Arc::new(api::Server {
        db: RwLock::new(Some(db::open().await)),
        // A new cluster was just backed up; a restored one is backed up on the first /sync.
        last_backup: Mutex::new((!restored).then(Instant::now)),
    });
    let app = openapi::server::new(server.clone()).layer(middleware::from_fn(logger::log_request));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();
    println!("listening on :{port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(sigterm())
        .await
        .unwrap();

    // Cloud Run sends SIGTERM before stopping the instance; archive pending changes to GCS.
    if let Err(e) = server.stop().await {
        eprintln!("stop on shutdown: {e}");
    }
}

async fn sigterm() {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .unwrap()
        .recv()
        .await;
}
