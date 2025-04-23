use serde_json::{json, Value};
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    // Start the bravesearch-mcp process with stdio transport
    let mut child = Command::new("target/debug/bravesearch-mcp")
        .arg("--api-key")
        .arg(std::env::var("BRAVE_API_KEY").expect("BRAVE_API_KEY must be set"))
        .arg("stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // Get handles to stdin and stdout
    let child_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let child_stdout = child.stdout.as_mut().expect("Failed to open stdout");

    // Create a JSON-RPC request to call brave_news_search for "trump" news
    let request = json!({
        "jsonrpc": "2.0",
        "method": "brave_news_search",
        "params": {
            "query": "trump",
            "count": 3,
            "country": "US",
            "search_lang": "en"
        },
        "id": 1
    });

    // Write the request to the process's stdin
    writeln!(child_stdin, "{}", request.to_string())?;
    
    // Read and print the response
    let mut buffer = [0; 8192];
    let read_bytes = child_stdout.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[0..read_bytes]);
    
    // Print the raw response
    println!("Raw response: {}", response);
    
    // Try to parse the response as JSON
    match serde_json::from_str::<Value>(&response) {
        Ok(json_response) => {
            println!("\nParsed JSON response:");
            if let Some(result) = json_response.get("result") {
                println!("{}", result);
            } else if let Some(error) = json_response.get("error") {
                println!("Error: {}", error);
            }
        },
        Err(e) => println!("Failed to parse JSON response: {}", e),
    }

    // Terminate the child process
    child.kill()?;
    
    Ok(())
}