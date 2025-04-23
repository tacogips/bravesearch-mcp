use crate::tools::bravesearch::BraveSearchRouter;
use anyhow::Result;
use rmcp::transport::stdio;
use rmcp::ServiceExt;

pub async fn run_stdio_server() -> Result<()> {
    // Create an instance of our search router
    let service = BraveSearchRouter::new();

    // Use the rust-sdk stdio transport implementation
    let server = service.serve(stdio()).await?;

    server.waiting().await?;
    Ok(())
}