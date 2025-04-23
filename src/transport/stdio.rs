use crate::tools::bravesearch::BraveSearchRouter;
use anyhow::Result;
use rmcp::transport::stdio;
use rmcp::ServiceExt;
use std::env;

pub async fn run_stdio_server() -> Result<()> {
    // Get API key from environment
    let api_key = env::var("BRAVE_API_KEY")
        .expect("BRAVE_API_KEY environment variable is required");
    
    // Create an instance of our search router with the API key
    let service = BraveSearchRouter::with_api_key(api_key);

    // Use the rust-sdk stdio transport implementation
    let server = service.serve(stdio()).await?;

    server.waiting().await?;
    Ok(())
}