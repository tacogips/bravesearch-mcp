use crate::tools::bravesearch::BraveSearchRouter;
use anyhow::Result;
use rmcp::transport::stdio;
use rmcp::ServiceExt;

pub async fn run_stdio_server(api_key: String) -> Result<()> {
    // Create an instance of our search router with the API key
    let service = BraveSearchRouter::new(api_key);

    // Use the rust-sdk stdio transport implementation
    let server = service.serve(stdio()).await?;

    server.waiting().await?;
    Ok(())
}