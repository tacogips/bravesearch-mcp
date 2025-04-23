use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::Mutex;

use rmcp::{model::*, schemars, tool, ServerHandler};

// Rate limiting configuration
const RATE_LIMIT_PER_SECOND: usize = 1;
const RATE_LIMIT_PER_MONTH: usize = 15000;

// Rate limiter
#[derive(Clone)]
struct RateLimiter {
    request_count: Arc<Mutex<RequestCount>>,
}

struct RequestCount {
    second: usize,
    month: usize,
    last_reset: Instant,
}

impl Default for RequestCount {
    fn default() -> Self {
        Self {
            second: 0,
            month: 0,
            last_reset: Instant::now(),
        }
    }
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
pub struct BraveSearchRouter {
    pub client: Client,
    rate_limiter: RateLimiter,
    api_key: String,
}

impl BraveSearchRouter {
    /// Create a new BraveSearchRouter with the required API key
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
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
impl BraveSearchRouter {
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
    ) -> String {
        let count = count.unwrap_or(10).min(20);
        let offset = offset.unwrap_or(0).min(9);

        match self.perform_web_search(&query, count, offset).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
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
    ) -> String {
        let count = count.unwrap_or(5).min(20);

        match self.perform_local_search(&query, count).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for BraveSearchRouter {
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