mod claude;
mod commands;
mod server;

use anyhow::Result;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "apprentice=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let apprentice_name =
        std::env::var("APPRENTICE_NAME").unwrap_or_else(|_| "unnamed".to_string());
    let port = std::env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());

    info!("Apprentice {} starting on port {}", apprentice_name, port);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().map_err(|e| {
        error!("Failed to parse address: {}", e);
        e
    })?;

    info!("Apprentice {} awakening on {}", apprentice_name, addr);

    info!("Creating apprentice server...");
    let apprentice = server::ApprenticeServer::new(apprentice_name);
    let apprentice_service = server::spells::apprentice_server::ApprenticeServer::new(apprentice);

    info!("Starting gRPC server...");

    // Set up graceful shutdown
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Received shutdown signal");
        let _ = shutdown_tx.send(());
    });

    Server::builder()
        .add_service(apprentice_service)
        .serve_with_shutdown(addr, async {
            shutdown_rx.await.ok();
            info!("Graceful shutdown initiated");
        })
        .await
        .map_err(|e| {
            error!("Apprentice server failed: {}", e);
            e
        })?;

    info!("Server shutting down");

    Ok(())
}
