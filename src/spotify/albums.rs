use serde::Deserialize;
use crate::utils::settings::Settings;
use curl::easy::Easy;
use std::io::Write;

#[derive(Deserialize)]
pub struct SpotifyTopResponse {
    pub items: Vec<TopItem>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub href: String,
    pub next: Option<String>,
    pub previous: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum TopItem {
    Track(Track),
    Artist(Artist),
}

#[derive(Deserialize)]
pub struct Track {
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
    pub is_playable: bool,
    pub name: String,
    pub popularity: u32,
    pub preview_url: Option<String>,
    pub track_number: u32,
    #[serde(rename = "type")]
    pub item_type: String,
    pub uri: String,
    pub is_local: bool,
    pub restrictions: Option<Restrictions>,
}

#[derive(Deserialize)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: u32,
    pub available_markets: Vec<String>,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<AlbumImage>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub uri: String,
    pub artists: Vec<Artist>,
    pub is_playable: Option<bool>,
}

#[derive(Deserialize)]
pub struct Artist {
    pub external_urls: ExternalUrls,
    pub followers: Option<Followers>,
    pub genres: Option<Vec<String>>,
    pub href: String,
    pub id: String,
    pub images: Option<Vec<AlbumImage>>,
    pub name: String,
    pub popularity: Option<u32>,
    #[serde(rename = "type")]
    pub item_type: String,
    pub uri: String,
}

#[derive(Deserialize)]
pub struct AlbumImage {
    pub url: String,
    pub height: u32,
    pub width: u32,
}

#[derive(Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Deserialize)]
pub struct ExternalIds {
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}

#[derive(Deserialize)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u32,
}

#[derive(Deserialize)]
pub struct Restrictions {
    pub reason: String,
}

pub async fn fetch_top_items(
    settings: &Settings, 
    access_token: &str, 
    item_type: &str,
    time_range: &str,
    offset: u32
) -> Result<SpotifyTopResponse, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.spotify.com/v1/me/top/{}?limit={}&offset={}&time_range={}",
        item_type, settings.limit, offset, time_range
    );
    
    let mut easy = Easy::new();
    let mut response_data = Vec::new();
    let mut headers = curl::easy::List::new();
    
    // Set headers
    headers.append(&format!("Authorization: Bearer {}", access_token))?;
    headers.append("Accept: */*")?;
    headers.append("User-Agent: curl/8.14.1")?;
    
    easy.url(&url)?;
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
            return Err(format!("Curl error: {}", e).into());
        }
    }
    
    let status_code = easy.response_code()?;
    let response_text = String::from_utf8(response_data)?;
    
    if status_code >= 200 && status_code < 300 {
        match serde_json::from_str::<SpotifyTopResponse>(&response_text) {
            Ok(top_items) => Ok(top_items),
            Err(e) => Err(e.into())
        }
    } else {
        let error_msg = match status_code {
            401 => "Authentication failed - your access token may be expired or invalid. Please re-authenticate.".to_string(),
            403 => {
                if response_text.contains("Insufficient client scope") {
                    "Insufficient permissions - your access token doesn't have the required scope. You need 'user-top-read' scope to access top items. Please re-authenticate with the correct scopes.".to_string()
                } else {
                    format!("Forbidden - {}", response_text)
                }
            },
            429 => "Rate limit exceeded - too many requests. Please wait a moment and try again.".to_string(),
            500..=599 => "Spotify API server error - please try again later.".to_string(),
            _ => format!("API request failed with status: {} - {}", status_code, response_text)
        };
        
        Err(error_msg.into())
    }
}

// Keep the old function for backward compatibility, but deprecate it
#[deprecated(note = "Use fetch_top_items instead")]
pub async fn fetch_albums(settings: &Settings, access_token: &str, offset: u32) -> Result<SpotifyTopResponse, Box<dyn std::error::Error>> {
    fetch_top_items(settings, access_token, "tracks", "medium_term", offset).await
}