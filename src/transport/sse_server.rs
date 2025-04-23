use anyhow::Result;
use rmcp::{Service, transport::sse_server::SseServer, ServerHandler, RoleServer};
use std::net::SocketAddr;
use tokio::task::JoinHandle;

pub async fn serve<S>(service: S, port: u16) -> Result<JoinHandle<Result<()>>>
where
    S: Service<RoleServer> + ServerHandler + Clone + Send + Sync + 'static,
{
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let sse_server = SseServer::serve(addr).await?;
    let cancellation_token = sse_server.with_service(move || service.clone());

    // Spawn a task that waits for Ctrl+C and then cancels the server
    let handle = tokio::spawn(async move {
        // Wait for Ctrl+C signal to gracefully shutdown
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::error!("Failed to listen for ctrl+c: {}", e);
        }
        
        // Cancel the server
        tracing::info!("Shutting down server...");
        cancellation_token.cancel();
        
        Ok(())
    });

    Ok(handle)
}