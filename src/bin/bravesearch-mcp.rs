use std::env;

use anyhow::{anyhow, Result};
use bravesearch_mcp::tools::BraveSearchRouter;
use bravesearch_mcp::transport::stdio;
use clap::{Parser, Subcommand};
use rmcp::ServiceExt;
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Parser)]
#[command(name = "bravesearch-mcp")]
#[command(about = "Brave Search MCP Server", long_about = None)]
struct Cli {
    /// Optional Brave API key, overrides BRAVE_API_KEY environment variable
    #[arg(short, long, env = "BRAVE_API_KEY")]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the Brave Search MCP server over stdio
    Stdio,
    /// Run the Brave Search MCP server over SSE
    Sse {
        /// Port to use for SSE server
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

fn get_api_key(cli_key: Option<String>) -> Result<String> {
    match cli_key {
        Some(key) => Ok(key),
        None => env::var("BRAVE_API_KEY").map_err(|_| {
            anyhow!("BRAVE_API_KEY environment variable is required when --api-key is not provided")
        }),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Get API key
    let api_key = get_api_key(cli.api_key)?;

    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env())
        .init();

    info!("Starting Brave Search MCP server");

    match cli.command {
        Commands::Stdio => {
            info!("Running in stdio mode");
            
            // Create the router with the API key
            let service = BraveSearchRouter::with_api_key(api_key);
            
            // Serve the router over stdio
            let server = service.serve(rmcp::transport::stdio()).await?;
            server.waiting().await?;
            
            Ok(())
        }
        Commands::Sse { port } => {
            info!("Running in SSE mode on port {}", port);
            
            // Create a service instance with the API key
            let service = BraveSearchRouter::with_api_key(api_key);
            
            // Configure and start the server
            let server = bravesearch_mcp::transport::sse_server::serve(service, port).await?;
            
            // Wait for server to complete
            server.await?;
            
            Ok(())
        }
    }
}