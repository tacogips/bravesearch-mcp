# Brave Search MCP

This is a Model Context Protocol (MCP) server that provides tools for web search and local search using the Brave Search API. It enables LLMs to search the web and find local businesses through a standardized interface.

## Features

- **brave_web_search**: Perform web searches using the Brave Search API
- **brave_local_search**: Find local businesses and places

## Prerequisites

You need a Brave Search API key to use this server. You can obtain one by visiting [Brave Search API](https://brave.com/search/api/).

## Installation

```bash
git clone https://github.com/tacogips/bravesearch-mcp.git
cd bravesearch-mcp
cargo build --release
```

## Running the Server

There are two ways to provide your Brave API key:

1. Set it as an environment variable:
   ```bash
   export BRAVE_API_KEY=your_api_key_here
   ```

2. Provide it directly as a command-line argument:
   ```bash
   cargo run --bin bravesearch-mcp --api-key your_api_key_here stdio
   ```

Choose the mode that suits your needs:

### STDIN/STDOUT Mode

This mode is useful when you want to pipe data directly to and from the server:

```bash
# Run in STDIN/STDOUT mode with environment variable
cargo run --bin bravesearch-mcp stdio

# Run in STDIN/STDOUT mode with command-line API key
cargo run --bin bravesearch-mcp --api-key your_api_key_here stdio
```

### HTTP/SSE Mode

Server-Sent Events (SSE) mode runs an HTTP server:

```bash
# Run in HTTP/SSE mode (default port: 3000)
cargo run --bin bravesearch-mcp sse

# Run in HTTP/SSE mode with custom port
cargo run --bin bravesearch-mcp sse --port 8080

# Run in HTTP/SSE mode with command-line API key
cargo run --bin bravesearch-mcp --api-key your_api_key_here sse
```

## Command-Line Options

The server supports the following command-line options:

```
USAGE:
    bravesearch-mcp [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -a, --api-key <API_KEY>    Brave API key, required if BRAVE_API_KEY environment variable is not set
    -h, --help                 Print help information

SUBCOMMANDS:
    help     Print this message or the help of the given subcommand(s)
    sse      Run the Brave Search MCP server over SSE
    stdio    Run the Brave Search MCP server over stdio
```

For the `sse` subcommand, you can also specify the port:

```
USAGE:
    bravesearch-mcp sse [OPTIONS]

OPTIONS:
    -p, --port <PORT>    Port to use for SSE server [default: 3000]
```

## Using the Example Client

An example client is included to demonstrate how to interact with the server:

```bash
# If you've set the BRAVE_API_KEY environment variable:
cargo run --example client

# Or, set it when running the example:
BRAVE_API_KEY=your_api_key_here cargo run --example client
```

The example client demonstrates:

1. STDIN/STDOUT communication with the server
2. HTTP/SSE communication
3. Making web search and local search requests

## Available Tools

The server provides the following tools:

### 1. `brave_web_search`

Performs web searches using the Brave Search API.

Parameters:

- `query` (required): The search query (max 400 chars, 50 words)
- `count` (optional): Number of results to return (1-20, default 10)
- `offset` (optional): Pagination offset (max 9, default 0)

Example:

```json
{
  "name": "brave_web_search",
  "arguments": {
    "query": "What is the Brave browser?",
    "count": 5
  }
}
```

### 2. `brave_local_search`

Searches for local businesses and places.

Parameters:

- `query` (required): The local search query (e.g., "pizza near Central Park")
- `count` (optional): Number of results to return (1-20, default 5)

Example:

```json
{
  "name": "brave_local_search",
  "arguments": {
    "query": "Coffee shops in Seattle",
    "count": 3
  }
}
```

## Implementation Notes

- The server implements rate limiting to adhere to Brave Search API restrictions
- Local search automatically falls back to web search if no local results are found
- Results for local searches include detailed business information including address, phone, ratings, etc.

## Rate Limits

The Brave Search API has the following rate limits:

- 1 request per second
- 15,000 requests per month

The server implements these rate limits to prevent exceeding the API quotas.

## MCP Protocol Integration

This server implements the Model Context Protocol (MCP) which allows it to be easily integrated with LLM clients that support the protocol. For more information about MCP, visit [the MCP repository](https://github.com/modelcontextprotocol/mcp).

## License

MIT License