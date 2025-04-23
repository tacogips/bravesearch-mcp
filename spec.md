# Brave Search MCP Specification

## Overview

This is a Model Context Protocol (MCP) server implementation that interfaces with the Brave Search API. It provides tools for performing web searches and local business searches through a standardized interface.

## API Requirements

- A Brave Search API key is required for operation
- The API key must be provided either via the `BRAVE_API_KEY` environment variable or the `--api-key` command-line argument

## Implementation Details

### Router Configuration

The main router class, `BraveSearchRouter`, is instantiated with the following parameters:

```rust
// Create a new BraveSearchRouter with the required API key
BraveSearchRouter::new(api_key: String)
```

### Rate Limiting

The implementation includes built-in rate limiting to adhere to Brave Search API restrictions:
- 1 request per second
- 15,000 requests per month

### Tools

#### 1. brave_web_search

Performs web searches using the Brave Search API.

Parameters:
- `query` (required): Search query (max 400 chars, 50 words)
- `count` (optional): Number of results (1-20, default 10)
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

#### 2. brave_local_search

Searches for local businesses and places using Brave's Local Search API.

Parameters:
- `query` (required): Local search query (e.g., "pizza near Central Park")
- `count` (optional): Number of results (1-20, default 5)

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

## Transport Options

The server supports two transport methods:
1. STDIN/STDOUT: For direct pipe communication
2. HTTP/SSE: For web-based clients with Server-Sent Events

## Reference Implementation

The original TypeScript reference implementation can be found at:
https://github.com/modelcontextprotocol/servers/blob/main/src/brave-search/index.ts
