use std::env;

use anyhow::Result;
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

#[tokio::main]
async fn main() -> Result<()> {
    // Check for API key
    if env::var("BRAVE_API_KEY").is_err() {
        eprintln!("Error: BRAVE_API_KEY environment variable is required");
        std::process::exit(1);
    }

    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(EnvFilter::from_default_env())
        .init();

    info!("Starting Brave Search MCP server");

    let cli = Cli::parse();

    match cli.command {
        Commands::Stdio => {
            info!("Running in stdio mode");
            stdio::run_stdio_server().await
        }
        Commands::Sse { port } => {
            info!("Running in SSE mode on port {}", port);
            
            // Create a service instance
            let service = BraveSearchRouter::new();
            
            // Configure and start the server
            let server = bravesearch_mcp::transport::sse_server::serve(service, port).await?;
            
            // Wait for server to complete
            server.await?;
            
            Ok(())
        }
    }
}