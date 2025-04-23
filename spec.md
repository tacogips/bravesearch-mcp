# Brave Search MCP Specification

## Overview

This is a Model Context Protocol (MCP) server implementation that interfaces with the Brave Search API. It provides tools for performing web searches, news searches, and local business searches through a standardized interface.

## API Requirements

- A Brave Search API key is required for operation
- The API key must be provided either via the `BRAVE_API_KEY` environment variable or the `--api-key` command-line argument
- Access to the Brave Search API (subscribe at https://api-dashboard.search.brave.com)

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

#### 2. brave_news_search

Searches for news articles using the Brave News Search API.

Parameters:
- `query` (required): News search query (max 400 chars, 50 words)
- `count` (optional): Number of results (1-50, default 20)
- `offset` (optional): Pagination offset (max 9, default 0)
- `country` (optional): Country code (default US)
  - Available options: ALL, AR, AU, AT, BE, BR, CA, CL, DK, FI, FR, DE, HK, IN, ID, IT, JP, KR, MY, MX, NL, NZ, NO, CN, PL, PT, PH, RU, SA, ZA, ES, SE, CH, TW, TR, GB, US
- `search_lang` (optional): Search language (default en)
  - Available options: ar, eu, bn, bg, ca, zh-hans, zh-hant, hr, cs, da, nl, en, en-gb, et, fi, fr, gl, de, gu, he, hi, hu, is, it, ja, kn, ko, lv, lt, ms, ml, mr, nb, pl, pt, pt-br, pa, ro, ru, sr, sk, sl, es, sv, ta, te, th, tr, uk, vi
- `freshness` (optional): Timeframe filter (h for hour, d for day, w for week, m for month, y for year)

Example:
```json
{
  "name": "brave_news_search",
  "arguments": {
    "query": "AI advancements",
    "count": 5,
    "country": "JP",
    "search_lang": "en",
    "freshness": "w"
  }
}
```

#### 3. brave_local_search

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

## API Documentation References

This implementation references the following Brave Search API documentation:

- Web Search API: https://api-dashboard.search.brave.com/app/documentation/web-search/get-started
- News Search API: https://api-dashboard.search.brave.com/app/documentation/news-search/get-started
- Local Search API: https://api-dashboard.search.brave.com/app/documentation/local-search/overview
- Country Codes: https://api-dashboard.search.brave.com/app/documentation/news-search/codes
- Parameters: https://api-dashboard.search.brave.com/app/documentation/news-search/query
