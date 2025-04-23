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

First, set your Brave API key as an environment variable:

```bash
export BRAVE_API_KEY=your_api_key_here
```

There are multiple ways to run the server:

### STDIN/STDOUT Mode

This mode is useful when you want to pipe data directly to and from the server:

```bash
# Run in STDIN/STDOUT mode
cargo run --bin bravesearch-mcp stdio
```

### HTTP/SSE Mode

Server-Sent Events (SSE) mode runs an HTTP server:

```bash
# Run in HTTP/SSE mode (default port: 3000)
cargo run --bin bravesearch-mcp sse

# Run in HTTP/SSE mode with custom port
cargo run --bin bravesearch-mcp sse --port 8080
```

## Using the Example Client

An example client is included to demonstrate how to interact with the server:

```bash
# Set your API key
export BRAVE_API_KEY=your_api_key_here

# Run the example client
cargo run --example client
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
