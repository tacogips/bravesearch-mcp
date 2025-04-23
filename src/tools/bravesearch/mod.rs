use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use rmcp::{model::*, schemars, tool, ServerHandler};

// Rate limiting configuration
const RATE_LIMIT_PER_SECOND: usize = 1;
const RATE_LIMIT_PER_MONTH: usize = 15000;

// Cache for documentation lookups to avoid repeated requests
#[derive(Clone)]
pub struct DocCache {
    cache: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

impl Default for DocCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DocCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.lock().await;
        cache.get(key).cloned()
    }

    pub async fn set(&self, key: String, value: String) {
        let mut cache = self.cache.lock().await;
        cache.insert(key, value);
    }
}

// Rate limiter
#[derive(Clone)]
struct RateLimiter {
    request_count: Arc<Mutex<RequestCount>>,
}

#[derive(Default)]
struct RequestCount {
    second: usize,
    month: usize,
    last_reset: Instant,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            request_count: Arc::new(Mutex::new(RequestCount {
                second: 0,
                month: 0,
                last_reset: Instant::now(),
            })),
        }
    }

    async fn check_rate_limit(&self) -> Result<()> {
        let mut req_count = self.request_count.lock().await;
        let now = Instant::now();
        
        if now.duration_since(req_count.last_reset) > Duration::from_secs(1) {
            req_count.second = 0;
            req_count.last_reset = now;
        }
        
        if req_count.second >= RATE_LIMIT_PER_SECOND || req_count.month >= RATE_LIMIT_PER_MONTH {
            return Err(anyhow!("Rate limit exceeded"));
        }
        
        req_count.second += 1;
        req_count.month += 1;
        
        Ok(())
    }
}

// Brave Search API Response Types
#[derive(Debug, Deserialize)]
struct BraveWebResult {
    title: String,
    description: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct BraveWeb {
    #[serde(default)]
    web: Option<BraveWebResults>,
    #[serde(default)]
    locations: Option<BraveLocationsResults>,
}

#[derive(Debug, Deserialize, Default)]
struct BraveWebResults {
    #[serde(default)]
    results: Vec<BraveWebResult>,
}

#[derive(Debug, Deserialize, Default)]
struct BraveLocationsResults {
    #[serde(default)]
    results: Vec<BraveLocationRef>,
}

#[derive(Debug, Deserialize)]
struct BraveLocationRef {
    id: String,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BravePoiResponse {
    results: Vec<BraveLocation>,
}

#[derive(Debug, Deserialize)]
struct BraveLocation {
    id: String,
    name: String,
    #[serde(default)]
    address: BraveAddress,
    #[serde(default)]
    coordinates: Option<BraveCoordinates>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    rating: Option<BraveRating>,
    #[serde(default)]
    opening_hours: Option<Vec<String>>,
    #[serde(default)]
    price_range: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct BraveAddress {
    #[serde(default)]
    street_address: Option<String>,
    #[serde(default)]
    address_locality: Option<String>,
    #[serde(default)]
    address_region: Option<String>,
    #[serde(default)]
    postal_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveCoordinates {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Deserialize)]
struct BraveRating {
    #[serde(default)]
    rating_value: Option<f64>,
    #[serde(default)]
    rating_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct BraveDescription {
    descriptions: std::collections::HashMap<String, String>,
}

#[derive(Clone)]
pub struct CargoDocRouter {
    pub client: Client,
    pub cache: DocCache,
    rate_limiter: RateLimiter,
    api_key: String,
}

impl CargoDocRouter {
    pub fn new() -> Self {
        // Get API key from environment
        let api_key = env::var("BRAVE_API_KEY")
            .expect("BRAVE_API_KEY environment variable is required");
        
        Self {
            client: Client::new(),
            cache: DocCache::new(),
            rate_limiter: RateLimiter::new(),
            api_key,
        }
    }

    async fn perform_web_search(&self, query: &str, count: usize, offset: usize) -> Result<String> {
        self.rate_limiter.check_rate_limit().await?;
        
        let url = reqwest::Url::parse_with_params(
            "https://api.search.brave.com/res/v1/web/search",
            &[
                ("q", query),
                ("count", &count.to_string()),
                ("offset", &offset.to_string()),
            ],
        )?;

        let response = self.client
            .get(url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Brave API error: {} {}\n{}", 
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or(""),
                response.text().await?
            ));
        }

        let data: BraveWeb = response.json().await?;
        let results = data.web
            .unwrap_or_default()
            .results
            .into_iter()
            .map(|result| {
                format!(
                    "Title: {}\nDescription: {}\nURL: {}",
                    result.title,
                    result.description,
                    result.url
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(results)
    }

    async fn perform_local_search(&self, query: &str, count: usize) -> Result<String> {
        self.rate_limiter.check_rate_limit().await?;
        
        let url = reqwest::Url::parse_with_params(
            "https://api.search.brave.com/res/v1/web/search",
            &[
                ("q", query),
                ("search_lang", "en"),
                ("result_filter", "locations"),
                ("count", &count.to_string()),
            ],
        )?;

        let response = self.client
            .get(url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Brave API error: {} {}\n{}", 
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or(""),
                response.text().await?
            ));
        }

        let web_data: BraveWeb = response.json().await?;
        let location_ids: Vec<String> = web_data
            .locations
            .unwrap_or_default()
            .results
            .into_iter()
            .map(|loc| loc.id)
            .collect();

        if location_ids.is_empty() {
            // Fall back to web search if no local results
            return self.perform_web_search(query, count, 0).await;
        }

        let pois_data = self.get_pois_data(&location_ids).await?;
        let desc_data = self.get_descriptions_data(&location_ids).await?;

        Ok(self.format_local_results(pois_data, desc_data))
    }

    async fn get_pois_data(&self, ids: &[String]) -> Result<BravePoiResponse> {
        self.rate_limiter.check_rate_limit().await?;
        
        let mut url = reqwest::Url::parse("https://api.search.brave.com/res/v1/local/pois")?;
        
        // Add all IDs as query parameters
        for id in ids {
            url.query_pairs_mut().append_pair("ids", id);
        }

        let response = self.client
            .get(url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Brave API error: {} {}\n{}", 
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or(""),
                response.text().await?
            ));
        }

        let pois_response: BravePoiResponse = response.json().await?;
        Ok(pois_response)
    }

    async fn get_descriptions_data(&self, ids: &[String]) -> Result<BraveDescription> {
        self.rate_limiter.check_rate_limit().await?;
        
        let mut url = reqwest::Url::parse("https://api.search.brave.com/res/v1/local/descriptions")?;
        
        // Add all IDs as query parameters
        for id in ids {
            url.query_pairs_mut().append_pair("ids", id);
        }

        let response = self.client
            .get(url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Brave API error: {} {}\n{}", 
                response.status().as_u16(),
                response.status().canonical_reason().unwrap_or(""),
                response.text().await?
            ));
        }

        let descriptions_data: BraveDescription = response.json().await?;
        Ok(descriptions_data)
    }

    fn format_local_results(&self, pois_data: BravePoiResponse, desc_data: BraveDescription) -> String {
        let results = pois_data.results.into_iter().map(|poi| {
            let address = [
                poi.address.street_address.unwrap_or_default(),
                poi.address.address_locality.unwrap_or_default(),
                poi.address.address_region.unwrap_or_default(),
                poi.address.postal_code.unwrap_or_default(),
            ]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(", ");

            let address_display = if address.is_empty() { "N/A" } else { &address };
            
            let rating = poi.rating.as_ref().and_then(|r| r.rating_value)
                .map(|val| val.to_string())
                .unwrap_or_else(|| "N/A".to_string());
            
            let rating_count = poi.rating.as_ref().and_then(|r| r.rating_count)
                .map(|val| val.to_string())
                .unwrap_or_else(|| "0".to_string());
            
            let hours = poi.opening_hours.unwrap_or_default().join(", ");
            let hours_display = if hours.is_empty() { "N/A" } else { &hours };
            
            let description = desc_data.descriptions.get(&poi.id)
                .cloned()
                .unwrap_or_else(|| "No description available".to_string());

            format!(
                "Name: {}\nAddress: {}\nPhone: {}\nRating: {} ({} reviews)\nPrice Range: {}\nHours: {}\nDescription: {}",
                poi.name,
                address_display,
                poi.phone.unwrap_or_else(|| "N/A".to_string()),
                rating,
                rating_count,
                poi.price_range.unwrap_or_else(|| "N/A".to_string()),
                hours_display,
                description
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");

        if results.is_empty() {
            "No local results found".to_string()
        } else {
            results
        }
    }
}

#[tool(tool_box)]
impl CargoDocRouter {
    #[tool(description = "Look up documentation for a Rust crate (returns markdown)")]
    async fn lookup_crate(
        &self,
        #[tool(param)]
        #[schemars(description = "The name of the crate to look up")]
        crate_name: String,

        #[tool(param)]
        #[schemars(description = "The version of the crate (optional, defaults to latest)")]
        version: Option<String>,
    ) -> String {
        // Check cache first
        let cache_key = if let Some(ver) = &version {
            format!("{}:{}", crate_name, ver)
        } else {
            crate_name.clone()
        };

        if let Some(doc) = self.cache.get(&cache_key).await {
            return doc;
        }

        // Construct the docs.rs URL for the crate
        let url = if let Some(ver) = version {
            format!("https://docs.rs/crate/{}/{}/", crate_name, ver)
        } else {
            format!("https://docs.rs/crate/{}/", crate_name)
        };

        // Fetch the documentation page
        let response = match self
            .client
            .get(&url)
            .header(
                "User-Agent",
                "CrateDocs/0.1.0 (https://github.com/d6e/bravesearch-mcp)",
            )
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => return format!("Failed to fetch documentation: {}", e),
        };

        if !response.status().is_success() {
            return format!(
                "Failed to fetch documentation. Status: {}",
                response.status()
            );
        }

        let html_body = match response.text().await {
            Ok(body) => body,
            Err(e) => return format!("Failed to read response body: {}", e),
        };

        // Convert HTML to markdown
        let markdown_body = html2md::parse_html(&html_body);

        // Cache the markdown result
        self.cache.set(cache_key, markdown_body.clone()).await;

        markdown_body
    }

    #[tool(description = "Performs a web search using the Brave Search API, ideal for general queries, news, articles, and online content.")]
    async fn brave_web_search(
        &self,
        #[tool(param)]
        #[schemars(description = "Search query (max 400 chars, 50 words)")]
        query: String,

        #[tool(param)]
        #[schemars(description = "Number of results (1-20, default 10)")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Pagination offset (max 9, default 0)")]
        offset: Option<usize>,
    ) -> Result<String> {
        let count = count.unwrap_or(10).min(20);
        let offset = offset.unwrap_or(0).min(9);

        self.perform_web_search(&query, count, offset).await
    }

    #[tool(description = "Searches for local businesses and places using Brave's Local Search API.")]
    async fn brave_local_search(
        &self,
        #[tool(param)]
        #[schemars(description = "Local search query (e.g. 'pizza near Central Park')")]
        query: String,

        #[tool(param)]
        #[schemars(description = "Number of results (1-20, default 5)")]
        count: Option<usize>,
    ) -> Result<String> {
        let count = count.unwrap_or(5).min(20);

        self.perform_local_search(&query, count).await
    }
}

#[tool(tool_box)]
impl ServerHandler for CargoDocRouter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Brave Search MCP Server for web and local search.".to_string(),
            ),
        }
    }
}