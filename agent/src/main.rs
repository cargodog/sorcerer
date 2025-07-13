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
            std::env::var("RUST_LOG").unwrap_or_else(|_| "agent=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let agent_name = std::env::var("AGENT_NAME").unwrap_or_else(|_| "unnamed".to_string());
    let port = std::env::var("GRPC_PORT").unwrap_or_else(|_| "50051".to_string());

    info!("Agent {} starting on port {}", agent_name, port);

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().map_err(|e| {
        error!("Failed to parse address: {}", e);
        e
    })?;

    info!("Agent {} awakening on {}", agent_name, addr);

    info!("Creating agent server...");
    let agent = server::AgentServer::new(agent_name);
    let agent_service = server::spells::agent_server::AgentServer::new(agent);

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
        .add_service(agent_service)
        .serve_with_shutdown(addr, async {
            shutdown_rx.await.ok();
            info!("Graceful shutdown initiated");
        })
        .await
        .map_err(|e| {
            error!("Agent server failed: {}", e);
            e
        })?;

    info!("Server shutting down");

    Ok(())
}
