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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CountryCode {
    ALL,
    AR,
    AU,
    AT,
    BE,
    BR,
    CA,
    CL,
    DK,
    FI,
    FR,
    DE,
    HK,
    IN,
    ID,
    IT,
    JP,
    KR,
    MY,
    MX,
    NL,
    NZ,
    NO,
    CN,
    PL,
    PT,
    PH,
    RU,
    SA,
    ZA,
    ES,
    SE,
    CH,
    TW,
    TR,
    GB,
    #[default]
    US,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LanguageCode {
    AR,
    EU,
    BN,
    BG,
    CA,
    ZhHans,
    ZhHant,
    HR,
    CS,
    DA,
    NL,
    #[default]
    EN,
    EnGb,
    ET,
    FI,
    FR,
    GL,
    DE,
    GU,
    HE,
    HI,
    HU,
    IS,
    IT,
    JA,
    KN,
    KO,
    LV,
    LT,
    MS,
    ML,
    MR,
    NB,
    PL,
    PT,
    PtBr,
    PA,
    RO,
    RU,
    SR,
    SK,
    SL,
    ES,
    SV,
    TA,
    TE,
    TH,
    TR,
    UK,
    VI,
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LanguageCode::ZhHans => write!(f, "zh-hans"),
            LanguageCode::ZhHant => write!(f, "zh-hant"),
            LanguageCode::EnGb => write!(f, "en-gb"),
            LanguageCode::PtBr => write!(f, "pt-br"),
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
            "zh-hans" => Ok(LanguageCode::ZhHans),
            "zh_hans" => Ok(LanguageCode::ZhHans),
            "zh-hant" => Ok(LanguageCode::ZhHant),
            "zh_hant" => Ok(LanguageCode::ZhHant),
            "hr" => Ok(LanguageCode::HR),
            "cs" => Ok(LanguageCode::CS),
            "da" => Ok(LanguageCode::DA),
            "nl" => Ok(LanguageCode::NL),
            "en" => Ok(LanguageCode::EN),
            "en-gb" => Ok(LanguageCode::EnGb),
            "en_gb" => Ok(LanguageCode::EnGb),
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
            "pt-br" => Ok(LanguageCode::PtBr),
            "pt_br" => Ok(LanguageCode::PtBr),
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    page_age: Option<String>,
    #[serde(rename = "page_fetched", default)]
    #[allow(dead_code)]
    page_fetched: Option<String>,
    #[serde(default)]
    thumbnail: Option<BraveNewsThumbnail>,
    #[serde(rename = "meta_url", default)]
    #[allow(dead_code)]
    meta_url: Option<BraveNewsMetaUrl>,
}

#[derive(Debug, Deserialize)]
struct BraveNewsThumbnail {
    #[serde(default)]
    src: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    original: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveNewsMetaUrl {
    #[serde(default)]
    #[allow(dead_code)]
    scheme: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    hostname: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    favicon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveLocationRef {
    id: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    latitude: f64,
    #[allow(dead_code)]
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
        freshness: Option<&str>,
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

        let results = data
            .results
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
                    breaking, result.title, result.description, result.url, age, thumbnail
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
        description = "Performs a web search using the Brave Search API, ideal for general queries, articles, and online content. This tool provides access to Brave's comprehensive web search index to find relevant websites, articles, and information across the internet. Results include title, description, and URL for each match to help answer factual questions and provide high-quality reference information."
    )]
    pub async fn brave_web_search(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Search query to find relevant web results. Limited to maximum 400 characters or 50 words. Use specific, concise queries for best results."
        )]
        query: String,

        #[tool(param)]
        #[schemars(
            description = "Number of results to return, between 1-20 (default 10). Higher values provide more comprehensive results but may include less relevant items."
        )]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(
            description = "Pagination offset for viewing additional results, maximum value 9 (default 0). Use incremental values to see more results beyond the initial set."
        )]
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
        description = "Searches for news articles using the Brave News Search API, ideal for current events, breaking news, and time-sensitive topics. This tool retrieves the latest news articles from a wide range of global news sources, providing timely information on current events, breaking news, and trending topics. Results include titles, descriptions, URLs, publication age, and often thumbnail images to provide comprehensive news coverage with real-time updates."
    )]
    pub async fn brave_news_search(
        &self,
        #[tool(param)]
        #[schemars(
            description = "News search query specifying the news topic or keywords to search for. Limited to maximum 400 characters or 50 words. Use clear, specific terms for more targeted news results."
        )]
        query: String,

        #[tool(param)]
        #[schemars(
            description = "Number of news articles to return, between 1-50 (default 20). Higher values provide more comprehensive coverage of a news topic."
        )]
        count: Option<usize>,

        #[tool(param)]
        #[schemars(
            description = "Pagination offset for viewing additional news results, maximum value 9 (default 0). Use with subsequent requests to see more news beyond the initial set."
        )]
        offset: Option<usize>,

        #[tool(param)]
        #[schemars(
            description = "Country code to filter news by geographic region. Options: ALL (worldwide), AR, AU, AT, BE, BR, CA, CL, DK, FI, FR, DE, HK, IN, ID, IT, JP, KR, MY, MX, NL, NZ, NO, CN, PL, PT, PH, RU, SA, ZA, ES, SE, CH, TW, TR, GB, US (default US). Use to get region-specific news coverage."
        )]
        country: Option<String>,

        #[tool(param)]
        #[schemars(
            description = "Search language for news articles. Options: ar, eu, bn, bg, ca, zh-hans, zh-hant, hr, cs, da, nl, en, en-gb, et, fi, fr, gl, de, gu, he, hi, hu, is, it, ja, kn, ko, lv, lt, ms, ml, mr, nb, pl, pt, pt-br, pa, ro, ru, sr, sk, sl, es, sv, ta, te, th, tr, uk, vi (default en). Determines the language of retrieved news articles."
        )]
        search_lang: Option<String>,

        #[tool(param)]
        #[schemars(
            description = "Timeframe filter to specify how recent the news should be. Use h (hour), d (day), w (week), m (month), or y (year) to control recency. Omit for all time periods. Most useful for filtering out older news when researching time-sensitive topics."
        )]
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

        match self
            .perform_news_search(
                &query,
                count,
                offset,
                country_code,
                lang_code,
                freshness_param,
            )
            .await
        {
            Ok(result) => result,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        description = "Searches for local businesses and places using Brave's Local Search API. This specialized search tool finds physical locations, businesses, landmarks, and points of interest based on geographic queries. It provides detailed information about each location including names, addresses, phone numbers, ratings, hours of operation, and descriptions, making it ideal for finding local services, restaurants, attractions, and other location-based information."
    )]
    pub async fn brave_local_search(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Local search query specifying what and where to search. Format should include both the category/business type and location (e.g., 'pizza near Central Park', 'coffee shops in Seattle', 'gas stations near me'). More specific queries yield better results."
        )]
        query: String,

        #[tool(param)]
        #[schemars(
            description = "Number of location results to return, between 1-20 (default 5). Higher values provide more options but may include less relevant locations. For popular searches in dense areas, higher values are recommended."
        )]
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
            instructions: Some(r#"Brave Search MCP Server providing access to Brave's web, news, and local search APIs.

TOOL USAGE EXAMPLES:

1. Web Search - For general information queries:
   ```
   brave_web_search(
     query: "rust programming language benefits",
     count: 5,  // Optional: Get 5 results (default: 10, max: 20)
     offset: 0  // Optional: Start from first result (default: 0, max: 9)
   )
   ```

2. News Search - For current events and breaking news:
   ```
   brave_news_search(
     query: "artificial intelligence developments",
     count: 10,            // Optional: Number of results (default: 20, max: 50)
     offset: 0,            // Optional: Pagination offset (default: 0, max: 9)
     country: "US",        // Optional: Country code (default: US)
     search_lang: "en",    // Optional: Language code (default: en)
     freshness: "d"        // Optional: Timeframe - d=day, w=week, m=month
   )
   ```

3. Local Search - For businesses and physical locations:
   ```
   brave_local_search(
     query: "pizza restaurants near Times Square",
     count: 5  // Optional: Number of results (default: 5, max: 20)
   )
   ```

All searches respect rate limits and provide formatted, readable results. Choose the appropriate tool based on the type of information needed."#.to_string()),
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
                Some("w".to_string()),
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
                None,
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

    // New unit tests
    #[test]
    fn test_country_code_from_str() {
        // Test valid country codes
        assert_eq!(CountryCode::from_str("US").unwrap(), CountryCode::US);
        assert_eq!(CountryCode::from_str("us").unwrap(), CountryCode::US);
        assert_eq!(CountryCode::from_str("JP").unwrap(), CountryCode::JP);
        assert_eq!(CountryCode::from_str("all").unwrap(), CountryCode::ALL);

        // Test invalid country code
        let invalid = CountryCode::from_str("ZZ");
        assert!(invalid.is_err());
        assert_eq!(invalid.unwrap_err(), "Unknown country code: ZZ");
    }

    #[test]
    fn test_language_code_from_str() {
        // Test valid language codes
        assert_eq!(LanguageCode::from_str("en").unwrap(), LanguageCode::EN);
        assert_eq!(LanguageCode::from_str("EN").unwrap(), LanguageCode::EN);
        assert_eq!(LanguageCode::from_str("en-gb").unwrap(), LanguageCode::EnGb);
        assert_eq!(
            LanguageCode::from_str("zh-hans").unwrap(),
            LanguageCode::ZhHans
        );

        // Test invalid language code
        let invalid = LanguageCode::from_str("xx");
        assert!(invalid.is_err());
        assert_eq!(invalid.unwrap_err(), "Unknown language code: xx");
    }

    #[test]
    fn test_country_code_display() {
        assert_eq!(CountryCode::US.to_string(), "us");
        assert_eq!(CountryCode::ALL.to_string(), "all");
        assert_eq!(CountryCode::JP.to_string(), "jp");
    }

    #[test]
    fn test_language_code_display() {
        assert_eq!(LanguageCode::EN.to_string(), "en");
        assert_eq!(LanguageCode::EnGb.to_string(), "en-gb");
        assert_eq!(LanguageCode::ZhHans.to_string(), "zh-hans");
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new();

        // First request should succeed
        assert!(limiter.check_rate_limit().await.is_ok());

        // Simulate reaching per-second limit
        {
            let mut count = limiter.request_count.lock().await;
            count.second = RATE_LIMIT_PER_SECOND;
        }

        // Next request should fail due to rate limit
        assert!(limiter.check_rate_limit().await.is_err());

        // Reset counter and test monthly limit
        {
            let mut count = limiter.request_count.lock().await;
            count.second = 0;
            count.month = RATE_LIMIT_PER_MONTH;
        }

        // Request should fail due to monthly limit
        assert!(limiter.check_rate_limit().await.is_err());
    }

    #[test]
    fn test_server_handler_info() {
        let router = BraveSearchRouter::new("test_key".to_string());
        let info = router.get_info();

        assert_eq!(info.protocol_version, ProtocolVersion::V_2024_11_05);
        assert!(info.instructions.is_some());
        assert!(info
            .instructions
            .unwrap()
            .contains("Brave Search MCP Server"));
    }
}
