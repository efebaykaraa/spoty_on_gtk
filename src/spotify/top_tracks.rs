use serde::Deserialize;
use curl::easy::Easy;
use std::io::Write;

const SPOTIFY_TOP_URL: &str = "https://api.spotify.com/v1/me/top";

#[derive(Debug, Clone)]
pub enum TopItemType {
    Artists,
    Tracks,
}

impl TopItemType {
    pub fn as_str(&self) -> &str {
        match self {
            TopItemType::Artists => "artists",
            TopItemType::Tracks => "tracks",
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimeRange {
    LongTerm,    // ≈ 1 year
    MediumTerm,  // ≈ 6 months (default)
    ShortTerm,   // ≈ 4 weeks
}

impl TimeRange {
    pub fn as_str(&self) -> &str {
        match self {
            TimeRange::LongTerm => "long_term",
            TimeRange::MediumTerm => "medium_term",
            TimeRange::ShortTerm => "short_term",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TopItemsResponse<T> {
    pub href: String,
    pub limit: u32,
    pub next: Option<String>,
    pub offset: u32,
    pub previous: Option<String>,
    pub total: u32,
    pub items: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct TopArtist {
    pub external_urls: ExternalUrls,
    pub followers: Followers,
    pub genres: Vec<String>,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub popularity: u32,
    #[serde(rename = "type")]
    pub artist_type: String,
    pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct TopTrack {
    pub album: Album,
    pub artists: Vec<Artist>,
    pub available_markets: Vec<String>,
    pub disc_number: u32,
    pub duration_ms: u64,
    pub explicit: bool,
    pub external_ids: ExternalIds,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub is_playable: Option<bool>,
    pub linked_from: Option<serde_json::Value>,
    pub restrictions: Option<Restrictions>,
    pub name: String,
    pub popularity: u32,
    pub preview_url: Option<String>,
    pub track_number: u32,
    #[serde(rename = "type")]
    pub track_type: String,
    pub uri: String,
    pub is_local: bool,
}

#[derive(Debug, Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Deserialize)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: u32,
    pub available_markets: Vec<String>,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    pub restrictions: Option<Restrictions>,
    #[serde(rename = "type")]
    pub album_type_field: String,
    pub uri: String,
    pub artists: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub artist_type: String,
    pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct ExternalIds {
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Restrictions {
    pub reason: String,
}

pub async fn fetch_top_items<T>(
    access_token: &str,
    item_type: TopItemType,
    time_range: Option<TimeRange>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<TopItemsResponse<T>, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de>,
{
    let url = format!("{}/{}", SPOTIFY_TOP_URL, item_type.as_str());
    
    let mut query_params = Vec::new();
    
    if let Some(range) = time_range {
        query_params.push(format!("time_range={}", range.as_str()));
    }
    
    if let Some(l) = limit {
        let clamped_limit = l.clamp(1, 50);
        query_params.push(format!("limit={}", clamped_limit));
    }
    
    if let Some(o) = offset {
        query_params.push(format!("offset={}", o));
    }
    
    let final_url = if query_params.is_empty() {
        url
    } else {
        format!("{}?{}", url, query_params.join("&"))
    };
    
    println!("Fetching top {} from: {}", item_type.as_str(), final_url);
    println!("Access token length: {}", access_token.len());
    
    let mut easy = Easy::new();
    let mut response_data = Vec::new();
    let mut headers = curl::easy::List::new();
    
    headers.append(&format!("Authorization: Bearer {}", access_token))?;
    headers.append("Accept: application/json")?;
    headers.append("Content-Type: application/json")?;
    
    easy.url(&final_url)?;
    easy.http_headers(headers)?;
    easy.timeout(std::time::Duration::from_secs(30))?;
    easy.connect_timeout(std::time::Duration::from_secs(15))?;
    
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            response_data.extend_from_slice(data);
            Ok(data.len())
        })?;
        
        if let Err(e) = transfer.perform() {
            println!("Curl error occurred: {}", e);
            return Err(format!("Curl error: {}", e).into());
        }
    }
    
    let status_code = easy.response_code()?;
    let response_text = String::from_utf8(response_data)?;
    
    println!("Response status: {}", status_code);
    println!("Response length: {} bytes", response_text.len());
    
    if response_text.len() > 0 {
        println!("Response preview: {}", &response_text[..std::cmp::min(200, response_text.len())]);
    }
    
    if status_code >= 200 && status_code < 300 {
        println!("Success! Parsing JSON response...");
        match serde_json::from_str::<TopItemsResponse<T>>(&response_text) {
            Ok(top_items) => {
                println!("Successfully parsed {} top {} items", top_items.items.len(), item_type.as_str());
                Ok(top_items)
            },
            Err(e) => {
                println!("JSON parsing error: {}", e);
                println!("Full response: {}", response_text);
                Err(e.into())
            }
        }
    } else {
        println!("API error - Status: {}, Response: {}", status_code, response_text);
        let error_msg = match status_code {
            401 => "Authentication failed - your access token may be expired or invalid. Please re-authenticate.".to_string(),
            403 => "Forbidden - insufficient permissions. Ensure you have the 'user-top-read' scope.".to_string(),
            429 => "Rate limit exceeded - too many requests. Please wait a moment and try again.".to_string(),
            500..=599 => "Spotify API server error - please try again later.".to_string(),
            _ => format!("API request failed with status: {} - {}", status_code, response_text)
        };
        
        Err(error_msg.into())
    }
}

pub async fn fetch_top_artists(
    access_token: &str,
    time_range: Option<TimeRange>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<TopItemsResponse<TopArtist>, Box<dyn std::error::Error>> {
    fetch_top_items(access_token, TopItemType::Artists, time_range, limit, offset).await
}

pub async fn fetch_top_tracks(
    access_token: &str,
    time_range: Option<TimeRange>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<TopItemsResponse<TopTrack>, Box<dyn std::error::Error>> {
    fetch_top_items(access_token, TopItemType::Tracks, time_range, limit, offset).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_string_conversion() {
        assert_eq!(TimeRange::LongTerm.as_str(), "long_term");
        assert_eq!(TimeRange::MediumTerm.as_str(), "medium_term");
        assert_eq!(TimeRange::ShortTerm.as_str(), "short_term");
    }

    #[test]
    fn test_top_item_type_string_conversion() {
        assert_eq!(TopItemType::Artists.as_str(), "artists");
        assert_eq!(TopItemType::Tracks.as_str(), "tracks");
    }
}
