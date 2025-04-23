use std::env;
use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

// Example client for BraveSearch-MCP
//
// This example demonstrates how to interact with the BraveSearch MCP server using both:
// 1. STDIO transport (Process-to-process communication via standard input/output)
// 2. HTTP/SSE transport (HTTP Server-Sent Events)
//
// The client demonstrates the proper MCP protocol sequence:
// - Send initialize request (with capabilities)
// - Process initialization response
// - Send initialized notification
// - List available tools
// - Make tool calls

// Simple example client for interacting with the server via stdin/stdout
async fn stdio_client(api_key: Option<&str>) -> Result<()> {
    // Prepare command to start the stdio-server
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(["run", "--bin", "bravesearch-mcp"]);
    
    // Add API key if provided
    if let Some(key) = api_key {
        cmd.arg("--api-key").arg(key);
    }
    
    // Add stdio subcommand
    cmd.arg("stdio");
    
    // Setup pipes for communication
    cmd.stdin(std::process::Stdio::piped())
       .stdout(std::process::Stdio::piped());
    
    // Start the process
    let mut child = cmd.spawn()?;

    let stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut stdin = io::BufWriter::new(stdin);
    let mut stdout = BufReader::new(stdout);

    // Send initialize request first
    let initialize_request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "BraveSearchExample",
                "version": "1.0.0"
            }
        },
        "id": 0
    });

    println!("Sending initialize request...");
    stdin
        .write_all(initialize_request.to_string().as_bytes())
        .await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read initialize response
    let mut init_response = String::new();
    stdout.read_line(&mut init_response).await?;
    println!("Initialize response: {:?}", init_response);

    // Send initialized notification
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });

    println!("Sending initialized notification...");
    stdin
        .write_all(initialized_notification.to_string().as_bytes())
        .await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Get list of available tools first
    let list_tools_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });

    println!("Sending request to list available tools...");
    stdin
        .write_all(list_tools_request.to_string().as_bytes())
        .await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read tools list response
    let mut tools_response = String::new();
    stdout.read_line(&mut tools_response).await?;
    println!("Tools list response: {:?}", tools_response);

    // Send a web search request
    let web_search_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "brave_web_search",
            "arguments": {
                "query": "What is the Brave browser?",
                "count": 3
            }
        },
        "id": 2
    });

    println!("Sending web search request...");
    stdin.write_all(web_search_request.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read the response
    let mut response = String::new();
    stdout.read_line(&mut response).await?;

    println!("Web search response: {:?}", response);
    let parsed: Value = serde_json::from_str(&response)?;
    println!(
        "Parsed response: {}",
        serde_json::to_string_pretty(&parsed)?
    );

    // Send a local search request
    let local_search_request = json!({
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "brave_local_search",
            "arguments": {
                "query": "Pizza near San Francisco",
                "count": 2
            }
        },
        "id": 3
    });

    println!("Sending local search request...");
    stdin.write_all(local_search_request.to_string().as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read the response
    let mut response = String::new();
    stdout.read_line(&mut response).await?;

    println!("Local search response: {:?}", response);
    let parsed: Value = serde_json::from_str(&response)?;
    println!(
        "Parsed response: {}",
        serde_json::to_string_pretty(&parsed)?
    );

    // Terminate the child process
    child.kill().await?;

    Ok(())
}

// Simple example client for interacting with the server via HTTP/SSE
async fn sse_client(api_key: Option<&str>, port: u16) -> Result<()> {
    println!("Connecting to HTTP/SSE server...");

    // Create HTTP client with timeout
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Prepare command to start the SSE server
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.args(["run", "--bin", "bravesearch-mcp"]);
    
    // Add API key if provided
    if let Some(key) = api_key {
        cmd.arg("--api-key").arg(key);
    }
    
    // Add sse subcommand with port
    cmd.arg("sse")
       .arg("--port")
       .arg(port.to_string());
    
    // Start the server on the specified port
    println!("Starting server on port {}...", port);
    let _server_handle = tokio::spawn(async move {
        match cmd.spawn() {
            Ok(mut child) => {
                match child.wait().await {
                    Ok(status) => println!("Server process exited with: {}", status),
                    Err(e) => println!("Error waiting for server: {}", e),
                }
            },
            Err(e) => println!("Failed to start server: {}", e),
        }
    });
    
    // Give the server some time to start
    println!("Waiting for server to start...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Generate a random session ID for testing
    let rand_num: u32 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let session_id = format!("test_session_{}", rand_num);
    
    println!("Using session ID: {}", session_id);
    let sse_url = format!("http://127.0.0.1:{}/sse?sessionId={}", port, session_id);

    // First send initialize request
    let init_request = json!({
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "BraveSearchExample",
                "version": "1.0.0"
            }
        },
        "id": 0
    });
    
    println!("Sending initialize request to SSE server...");
    let init_response = match client.post(&sse_url).json(&init_request).send().await {
        Ok(resp) => resp,
        Err(e) => {
            println!("Failed to send initialize request: {}", e);
            println!("\nIMPORTANT: HTTP/SSE transport requires special handling.");
            println!("The server expects EventSource connections, not regular HTTP requests.");
            
            // Try to abort the server process to clean up
            tokio::spawn(async {
                let _ = tokio::process::Command::new("pkill")
                    .args(["-f", &format!("bravesearch-mcp sse --port {}", port)])
                    .status()
                    .await;
            });
            
            return Ok(());
        }
    };

    println!("\n--- IMPLEMENTING A PROPER SSE CLIENT ---");
    println!("For a complete HTTP/SSE client implementation, you would need to:");
    println!("1. Use a library that supports SSE (EventSource) connections");
    println!("2. Establish a persistent SSE connection to /sse?sessionId=<id>");
    println!("3. Listen for events on that connection and parse them as JSON-RPC responses");
    println!("4. Send requests via HTTP POST to the same endpoint");
    println!("5. Match request IDs with response IDs to correlate requests and responses");
    
    // Clean up server process
    println!("\nCleaning up server process...");
    tokio::spawn(async move {
        let _ = tokio::process::Command::new("pkill")
            .args(["-f", &format!("bravesearch-mcp sse --port {}", port)])
            .status()
            .await;
    });

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Try to get API key from environment or let user know it's needed
    let api_key = match env::var("BRAVE_API_KEY") {
        Ok(key) => Some(key),
        Err(_) => {
            println!("Note: BRAVE_API_KEY environment variable is not set.");
            println!("You can either set it or provide the API key via --api-key parameter to the server.");
            None
        }
    };

    println!("Brave Search MCP Server Client Example");
    println!("--------------------------------------");

    // Run STDIO client test
    println!("\n1. Testing STDIN/STDOUT client:");
    if let Err(e) = stdio_client(api_key.as_deref()).await {
        println!("Error in STDIN/STDOUT client: {}", e);
    }

    // Run SSE client test
    println!("\n2. Testing HTTP/SSE client:");
    if let Err(e) = sse_client(api_key.as_deref(), 3000).await {
        println!("Error in HTTP/SSE client: {}", e);
    }

    Ok(())
}