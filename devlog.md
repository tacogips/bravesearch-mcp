# Development Log

This document tracks the development history, design decisions, and implementation details of the Brave Search MCP project.

To update this file:
1. Add new entries at the top of the "Recent Changes" section with today's date
2. Provide a concise summary of your changes
3. Document any design decisions or implementation details
4. Update relevant sections elsewhere in the document if needed

## Project Overview

The Brave Search MCP server provides a Model Context Protocol interface to the Brave Search API, allowing LLMs to perform web searches and local business searches through a standardized API.

## Architecture

The project follows a clean architecture with the following components:

1. **CLI Interface**: Handles command-line arguments and environment variables
2. **BraveSearchRouter**: Core class that handles request routing and API communication
3. **Transport Layers**: Support for both stdio and HTTP/SSE communication
4. **Rate Limiter**: Built-in mechanism to enforce Brave Search API rate limits

## Implementation Details

### API Key Management

- API keys are required and can be provided via:
  - `BRAVE_API_KEY` environment variable
  - `--api-key` command line argument
- The `BraveSearchRouter::new(api_key: String)` constructor takes the API key directly
- No environment variables are accessed within the router itself, making it more testable

### Rate Limiting

Rate limiting adheres to Brave Search API restrictions:
- 1 request per second
- 15,000 requests per month

Implementation uses a simple in-memory counter with Mutex for thread safety.

### Error Handling

- Error handling uses the `anyhow` crate for flexible error management
- Tool functions transform results to strings with error messages for user-friendly responses

## Recent Changes

### 2025-04-23: API Key Handling Refactoring

- Removed `with_api_key()` method from `BraveSearchRouter`
- Modified `BraveSearchRouter::new()` to take an `api_key` parameter directly
- Removed all usages of environment variable handling with `env::var`
- Changed the CLI parameter from `Option<String>` to `String` with `required = true`
- Simplified API key handling using clap's built-in environment variable support
- Updated documentation to reflect these changes

These changes simplify the code by:
1. Centralizing API key validation at the CLI level
2. Leveraging clap's built-in environment variable support
3. Making the router more testable by removing direct environment dependencies
4. Clarifying that the API key is a required parameter

## Roadmap

Potential future enhancements:
- Add support for more Brave Search API features
- Implement caching to reduce API usage
- Create a simple web UI for testing
- Add more comprehensive error handling and retry logic