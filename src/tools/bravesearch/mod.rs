use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use tokio::sync::Mutex;

use rmcp::{model::*, schemars, tool, ServerHandler};

// Rate limiting configuration
const RATE_LIMIT_PER_SECOND: usize = 1;
const RATE_LIMIT_PER_MONTH: usize = 15000;

// Country codes for Brave Search API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CountryCode {
    ALL, AR, AU, AT, BE, BR, CA, CL, DK, FI, FR, DE, HK, IN, ID, IT, JP, 
    KR, MY, MX, NL, NZ, NO, CN, PL, PT, PH, RU, SA, ZA, ES, SE, CH, TW, 
    TR, GB, US,
}

impl Default for CountryCode {
    fn default() -> Self {
        CountryCode::US
    }
}

impl fmt::Display for CountryCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use Debug representation which outputs the enum variant name
        let s = format!("{:?}", self).to_lowercase();
        // Special case for ALL which should be lowercase
        if s == "all" {
            write!(f, "all")
        } else {
            write!(f, "{}", s)
        }
    }
}

impl FromStr for CountryCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ALL" => Ok(CountryCode::ALL),
            "AR" => Ok(CountryCode::AR),
            "AU" => Ok(CountryCode::AU),
            "AT" => Ok(CountryCode::AT),
            "BE" => Ok(CountryCode::BE),
            "BR" => Ok(CountryCode::BR),
            "CA" => Ok(CountryCode::CA),
            "CL" => Ok(CountryCode::CL),
            "DK" => Ok(CountryCode::DK),
            "FI" => Ok(CountryCode::FI),
            "FR" => Ok(CountryCode::FR),
            "DE" => Ok(CountryCode::DE),
            "HK" => Ok(CountryCode::HK),
            "IN" => Ok(CountryCode::IN),
            "ID" => Ok(CountryCode::ID),
            "IT" => Ok(CountryCode::IT),
            "JP" => Ok(CountryCode::JP),
            "KR" => Ok(CountryCode::KR),
            "MY" => Ok(CountryCode::MY),
            "MX" => Ok(CountryCode::MX),
            "NL" => Ok(CountryCode::NL),
            "NZ" => Ok(CountryCode::NZ),
            "NO" => Ok(CountryCode::NO),
            "CN" => Ok(CountryCode::CN),
            "PL" => Ok(CountryCode::PL),
            "PT" => Ok(CountryCode::PT),
            "PH" => Ok(CountryCode::PH),
            "RU" => Ok(CountryCode::RU),
            "SA" => Ok(CountryCode::SA),
            "ZA" => Ok(CountryCode::ZA),
            "ES" => Ok(CountryCode::ES),
            "SE" => Ok(CountryCode::SE),
            "CH" => Ok(CountryCode::CH),
            "TW" => Ok(CountryCode::TW),
            "TR" => Ok(CountryCode::TR),
            "GB" => Ok(CountryCode::GB),
            "US" => Ok(CountryCode::US),
            _ => Err(format!("Unknown country code: {}", s)),
        }
    }
}

// Language codes for Brave Search API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageCode {
    AR, EU, BN, BG, CA, ZH_HANS, ZH_HANT, HR, CS, DA, NL, EN, EN_GB, 
    ET, FI, FR, GL, DE, GU, HE, HI, HU, IS, IT, JA, KN, KO, LV, LT, 
    MS, ML, MR, NB, PL, PT, PT_BR, PA, RO, RU, SR, SK, SL, ES, SV, TA, 
    TE, TH, TR, UK, VI,
}

impl Default for LanguageCode {
    fn default() -> Self {
        LanguageCode::EN
    }
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageCode::ZH_HANS => write!(f, "zh-hans"),
            LanguageCode::ZH_HANT => write!(f, "zh-hant"),
            LanguageCode::EN_GB => write!(f, "en-gb"),
            LanguageCode::PT_BR => write!(f, "pt-br"),
            _ => {
                let s = format!("{:?}", self).to_lowercase();
                write!(f, "{}", s)
            }
        }
    }
}

impl FromStr for LanguageCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ar" => Ok(LanguageCode::AR),
            "eu" => Ok(LanguageCode::EU),
            "bn" => Ok(LanguageCode::BN),
            "bg" => Ok(LanguageCode::BG),
            "ca" => Ok(LanguageCode::CA),
            "zh-hans" => Ok(LanguageCode::ZH_HANS),
            "zh_hans" => Ok(LanguageCode::ZH_HANS),
            "zh-hant" => Ok(LanguageCode::ZH_HANT),
            "zh_hant" => Ok(LanguageCode::ZH_HANT),
            "hr" => Ok(LanguageCode::HR),
            "cs" => Ok(LanguageCode::CS),
            "da" => Ok(LanguageCode::DA),
            "nl" => Ok(LanguageCode::NL),
            "en" => Ok(LanguageCode::EN),
            "en-gb" => Ok(LanguageCode::EN_GB),
            "en_gb" => Ok(LanguageCode::EN_GB),
            "et" => Ok(LanguageCode::ET),
            "fi" => Ok(LanguageCode::FI),
            "fr" => Ok(LanguageCode::FR),
            "gl" => Ok(LanguageCode::GL),
            "de" => Ok(LanguageCode::DE),
            "gu" => Ok(LanguageCode::GU),
            "he" => Ok(LanguageCode::HE),
            "hi" => Ok(LanguageCode::HI),
            "hu" => Ok(LanguageCode::HU),
            "is" => Ok(LanguageCode::IS),
            "it" => Ok(LanguageCode::IT),
            "ja" => Ok(LanguageCode::JA),
            "kn" => Ok(LanguageCode::KN),
            "ko" => Ok(LanguageCode::KO),
            "lv" => Ok(LanguageCode::LV),
            "lt" => Ok(LanguageCode::LT),
            "ms" => Ok(LanguageCode::MS),
            "ml" => Ok(LanguageCode::ML),
            "mr" => Ok(LanguageCode::MR),
            "nb" => Ok(LanguageCode::NB),
            "pl" => Ok(LanguageCode::PL),
            "pt" => Ok(LanguageCode::PT),
            "pt-br" => Ok(LanguageCode::PT_BR),
            "pt_br" => Ok(LanguageCode::PT_BR),
            "pa" => Ok(LanguageCode::PA),
            "ro" => Ok(LanguageCode::RO),
            "ru" => Ok(LanguageCode::RU),
            "sr" => Ok(LanguageCode::SR),
            "sk" => Ok(LanguageCode::SK),
            "sl" => Ok(LanguageCode::SL),
            "es" => Ok(LanguageCode::ES),
            "sv" => Ok(LanguageCode::SV),
            "ta" => Ok(LanguageCode::TA),
            "te" => Ok(LanguageCode::TE),
            "th" => Ok(LanguageCode::TH),
            "tr" => Ok(LanguageCode::TR),
            "uk" => Ok(LanguageCode::UK),
            "vi" => Ok(LanguageCode::VI),
            _ => Err(format!("Unknown language code: {}", s)),
        }
    }
}

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
struct BraveSearchResponse {
    #[serde(rename = "type")]
    response_type: String,
    #[serde(default)]
    web: Option<BraveWebResults>,
    #[serde(default)]
    locations: Option<BraveLocationsResults>,
    // News search API returns results directly at top level
    #[serde(default)]
    results: Vec<BraveNewsResult>,
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

// This is kept for backwards compatibility but not actually used anymore
#[derive(Debug, Deserialize, Default)]
struct BraveNewsResults {
    #[serde(default)]
    results: Vec<BraveNewsResult>,
}

#[derive(Debug, Deserialize)]
struct BraveNewsResult {
    title: String,
    description: String,
    url: String,
    #[serde(default)]
    age: Option<String>,
    #[serde(default)]
    breaking: Option<bool>,
    #[serde(rename = "page_age", default)]
    page_age: Option<String>,
    #[serde(rename = "page_fetched", default)]
    page_fetched: Option<String>,
    #[serde(default)]
    thumbnail: Option<BraveNewsThumbnail>,
    #[serde(rename = "meta_url", default)]
    meta_url: Option<BraveNewsMetaUrl>,
}

#[derive(Debug, Deserialize)]
struct BraveNewsThumbnail {
    #[serde(default)]
    src: Option<String>,
    #[serde(default)]
    original: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveNewsMetaUrl {
    #[serde(default)]
    scheme: Option<String>,
    #[serde(default)]
    hostname: Option<String>,
    #[serde(default)]
    favicon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveLocationRef {
    id: String,
    #[serde(rename = "type")]
    location_type: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    coordinates: Option<Vec<f64>>,
    #[serde(default)]
    postal_address: Option<BravePostalAddress>,
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

#[derive(Debug, Deserialize, Default)]
struct BravePostalAddress {
    #[serde(default)]
    country: Option<String>,
    #[serde(default, rename = "postalCode")]
    postal_code: Option<String>,
    #[serde(default, rename = "streetAddress")]
    street_address: Option<String>,
    #[serde(default, rename = "addressLocality")]
    address_locality: Option<String>,
    #[serde(default, rename = "addressRegion")]
    address_region: Option<String>,
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
        // Create a client with default settings
        // The reqwest client automatically handles gzip responses by default
        // as long as the appropriate feature is enabled in Cargo.toml
        Self {
            client: Client::new(),
            rate_limiter: RateLimiter::new(),
            api_key,
        }
    }

    async fn perform_news_search(
        &self, 
        query: &str, 
        count: usize, 
        offset: usize, 
        country: Option<CountryCode>, 
        search_lang: Option<LanguageCode>, 
        freshness: Option<&str>
    ) -> Result<String> {
        self.rate_limiter.check_rate_limit().await?;

        // Build URL with query parameters
        let country_code = country.unwrap_or_default().to_string();
        let language_code = search_lang.unwrap_or_default().to_string();
        
        let mut params = vec![
            ("q", query.to_string()),
            ("count", count.to_string()),
            ("offset", offset.to_string()),
            ("country", country_code),
            ("search_lang", language_code),
            ("spellcheck", "1".to_string()),
        ];

        // Add optional parameters
        if let Some(freshness_val) = freshness {
            params.push(("freshness", freshness_val.to_string()));
        }

        let url = reqwest::Url::parse_with_params(
            "https://api.search.brave.com/res/v1/news/search",
            &params,
        )?;
        
        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let reason = response.status().canonical_reason().unwrap_or("");
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Brave API error: {} {}\n{}",
                status_code,
                reason,
                error_text
            ));
        }

        // Get response body as text
        let response_text = response.text().await?;
        
        // Parse the JSON
        let data = match serde_json::from_str::<BraveSearchResponse>(&response_text) {
            Ok(parsed) => parsed,
            Err(e) => {
                return Ok(format!("Failed to parse API response: {}", e));
            }
        };
        
        if data.results.is_empty() {
            return Ok("No news results found (empty results array)".to_string());
        }
        
        let results = data.results
            .iter() // Use iter() instead of into_iter() for shared references
            .map(|result| {
                let breaking = if result.breaking.unwrap_or(false) {
                    "[BREAKING] "
                } else {
                    ""
                };
                
                let age = result.age.as_deref().unwrap_or("Unknown");
                
                let thumbnail = match &result.thumbnail {
                    Some(thumb) => match &thumb.src {
                        Some(src) => format!("\nThumbnail: {}", src),
                        None => "".to_string(),
                    },
                    None => "".to_string(),
                };
                
                format!(
                    "{}Title: {}\nDescription: {}\nURL: {}\nAge: {}{}", 
                    breaking,
                    result.title, 
                    result.description, 
                    result.url,
                    age,
                    thumbnail
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(results)
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

        let response = self
            .client
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

        // With the gzip feature enabled, reqwest will automatically handle decompression
        let data: BraveSearchResponse = response.json().await?;
        let results = data
            .web
            .unwrap_or_default()
            .results
            .into_iter()
            .map(|result| {
                format!(
                    "Title: {}\nDescription: {}\nURL: {}",
                    result.title, result.description, result.url
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(results)
    }

    async fn perform_local_search(&self, query: &str, count: usize) -> Result<String> {
        self.rate_limiter.check_rate_limit().await?;

        // Use appropriate Local Search API endpoint and params
        let url = reqwest::Url::parse_with_params(
            "https://api.search.brave.com/res/v1/web/search",
            &[
                ("q", query),
                ("search_lang", "en"),
                ("result_filter", "locations"),
                ("count", &count.to_string()),
            ],
        )?;

        let response = self
            .client
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

        // Parse the response using the new BraveSearchResponse structure
        let search_data: BraveSearchResponse = response.json().await?;

        // Extract location references from the search response
        let location_refs = match &search_data.locations {
            Some(locations) => &locations.results,
            None => {
                // Fall back to web search if no local results
                return self.perform_web_search(query, count, 0).await;
            }
        };

        if location_refs.is_empty() {
            // Fall back to web search if no local results
            return self.perform_web_search(query, count, 0).await;
        }

        // Extract only the IDs for the POI data lookup
        let location_ids: Vec<String> = location_refs.iter().map(|loc| loc.id.clone()).collect();

        // Format results directly from location references if possible
        let mut results = Vec::new();

        for loc_ref in location_refs {
            let mut result_parts = Vec::new();

            // Try to use data directly from the search results first
            if let Some(title) = &loc_ref.title {
                result_parts.push(format!("Name: {}", title));
            }

            // Format address if available
            if let Some(address) = &loc_ref.postal_address {
                let address_parts = vec![
                    address.street_address.as_deref().unwrap_or(""),
                    address.address_locality.as_deref().unwrap_or(""),
                    address.address_region.as_deref().unwrap_or(""),
                    address.postal_code.as_deref().unwrap_or(""),
                    address.country.as_deref().unwrap_or(""),
                ];

                let address_str = address_parts
                    .into_iter()
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");

                if !address_str.is_empty() {
                    result_parts.push(format!("Address: {}", address_str));
                }
            }

            // Add coordinates if available
            if let Some(coords) = &loc_ref.coordinates {
                if coords.len() >= 2 {
                    result_parts.push(format!("Coordinates: {}, {}", coords[0], coords[1]));
                }
            }

            // Add the ID for reference
            result_parts.push(format!("ID: {}", loc_ref.id));

            results.push(result_parts.join("\n"));
        }

        // If we have basic information, return it
        if !results.is_empty() {
            return Ok(results.join("\n---\n"));
        }

        // Fall back to the old method of getting detailed POI data
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

        let response = self
            .client
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

        let mut url =
            reqwest::Url::parse("https://api.search.brave.com/res/v1/local/descriptions")?;

        // Add all IDs as query parameters
        for id in ids {
            url.query_pairs_mut().append_pair("ids", id);
        }

        let response = self
            .client
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

    fn format_local_results(
        &self,
        pois_data: BravePoiResponse,
        desc_data: BraveDescription,
    ) -> String {
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
    #[tool(
        description = "Performs a web search using the Brave Search API, ideal for general queries, articles, and online content."
    )]
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
    
    #[tool(
        description = "Searches for news articles using the Brave News Search API, ideal for current events, breaking news, and time-sensitive topics."
    )]
    async fn brave_news_search(
        &self,
        #[tool(param)]
        #[schemars(description = "News search query (max 400 chars, 50 words)")]
        query: String,

        #[tool(param)]
        #[schemars(description = "Number of results (1-50, default 20)")]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(description = "Pagination offset (max 9, default 0)")]
        offset: Option<usize>,
        
        #[tool(param)]
        #[schemars(description = "Country code (ALL, AR, AU, AT, BE, BR, CA, CL, DK, FI, FR, DE, HK, IN, ID, IT, JP, KR, MY, MX, NL, NZ, NO, CN, PL, PT, PH, RU, SA, ZA, ES, SE, CH, TW, TR, GB, US; default US)")]
        country: Option<String>,
        
        #[tool(param)]
        #[schemars(description = "Search language (ar, eu, bn, bg, ca, zh-hans, zh-hant, hr, cs, da, nl, en, en-gb, et, fi, fr, gl, de, gu, he, hi, hu, is, it, ja, kn, ko, lv, lt, ms, ml, mr, nb, pl, pt, pt-br, pa, ro, ru, sr, sk, sl, es, sv, ta, te, th, tr, uk, vi; default en)")]
        search_lang: Option<String>,
        
        #[tool(param)]
        #[schemars(description = "Timeframe filter (h for hour, d for day, w for week, m for month, y for year)")]
        freshness: Option<String>,
    ) -> String {
        let count = count.unwrap_or(20).min(50);
        let offset = offset.unwrap_or(0).min(9);
        
        // Parse country code if provided
        let country_code = match country {
            Some(c) => match CountryCode::from_str(&c) {
                Ok(code) => Some(code),
                Err(e) => return format!("Error parsing country code: {}", e),
            },
            None => None,
        };
        
        // Parse language code if provided
        let lang_code = match search_lang {
            Some(l) => match LanguageCode::from_str(&l) {
                Ok(code) => Some(code),
                Err(e) => return format!("Error parsing language code: {}", e),
            },
            None => None,
        };
        
        let freshness_param = freshness.as_deref();

        match self.perform_news_search(&query, count, offset, country_code, lang_code, freshness_param).await {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        description = "Searches for local businesses and places using Brave's Local Search API."
    )]
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
            instructions: Some("Brave Search MCP Server for web, news, and local search.".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_brave_search_apis() {
        // Skip the test if API key is not set in environment
        let api_key = std::env::var("BRAVE_API_KEY").unwrap_or_else(|_| {
            eprintln!("BRAVE_API_KEY environment variable not set, skipping test");
            String::from("dummy_key")
        });

        // Only run this test if we have a real API key
        if api_key == "dummy_key" {
            // Skip the test
            return;
        }

        // Create a BraveSearchRouter with the API key
        let router = BraveSearchRouter::new(api_key);

        // Test 1: Web Search
        let web_result = router
            .brave_web_search("Rust programming language".to_string(), Some(3), None)
            .await;
            
        println!("Web search result: {}", web_result);
        assert!(!web_result.is_empty());
        assert!(web_result.contains("Rust"));

        // Test 2: News Search with country and language
        let news_result = router
            .brave_news_search(
                "technology".to_string(), 
                Some(3), 
                None, 
                Some("JP".to_string()), 
                Some("en".to_string()), 
                Some("w".to_string())
            )
            .await;
            
        println!("News search result (JP, en): {}", news_result);
        assert!(!news_result.is_empty());
        assert!(news_result != "No news results found");
        assert!(!news_result.starts_with("Error parsing"));

        // Test 3: Local Search
        let local_result = router
            .brave_local_search("coffee shop".to_string(), Some(2))
            .await;
            
        println!("Local search result: {}", local_result);
        assert!(!local_result.is_empty());
    }
    
    #[tokio::test]
    async fn test_news_search_with_query() {
        // Skip the test if API key is not set in environment
        let api_key = std::env::var("BRAVE_API_KEY").unwrap_or_else(|_| {
            eprintln!("BRAVE_API_KEY environment variable not set, skipping test");
            String::from("dummy_key")
        });

        // Only run this test if we have a real API key
        if api_key == "dummy_key" {
            // Skip the test
            return;
        }

        // Create a BraveSearchRouter with the API key
        let router = BraveSearchRouter::new(api_key);

        // Search for current news with US country code and English language
        // Use "news" as a generic query that should always return results
        let news_result = router
            .brave_news_search(
                "news".to_string(),
                Some(3),
                None,
                Some("US".to_string()),
                Some("en".to_string()),
                None
            )
            .await;
            
        println!("News search result: {}", news_result);
        
        // Verify we got results
        assert!(!news_result.is_empty());
        assert!(news_result != "No news results found");
        assert!(!news_result.starts_with("Error parsing"));
        
        // Print the API response details
        println!("\nNews search API response received successfully!");
    }
}
