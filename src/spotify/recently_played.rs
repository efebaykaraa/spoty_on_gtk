use serde::Deserialize;
use crate::utils::settings::Settings;
use curl::easy::Easy;
use std::io::Write;

#[derive(Deserialize)]
pub struct RecentlyPlayedResponse {
    pub href: String,
    pub limit: u32,
    pub next: Option<String>,
    pub cursors: Cursors,
    pub total: Option<u32>,
    pub items: Vec<RecentlyPlayedItem>,
}

#[derive(Deserialize)]
pub struct Cursors {
    pub after: String,
    pub before: String,
}

#[derive(Deserialize)]
pub struct RecentlyPlayedItem {
    pub track: Track,
    pub played_at: String,
    pub context: Option<PlayContext>,
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
    pub is_playable: Option<bool>,
    pub name: String,
    pub popularity: u32,
    pub preview_url: Option<String>,
    pub track_number: u32,
    #[serde(rename = "type")]
    pub track_type: String,
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

#[derive(Deserialize)]
pub struct Artist {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub artist_type: String,
    pub uri: String,
}

#[derive(Deserialize)]
pub struct Image {
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
pub struct Restrictions {
    pub reason: String,
}

#[derive(Deserialize)]
pub struct PlayContext {
    #[serde(rename = "type")]
    pub context_type: String,
    pub href: String,
    pub external_urls: ExternalUrls,
    pub uri: String,
}

pub async fn fetch_recently_played(
    access_token: &str,
    client_token: &str,
) -> Result<RecentlyPlayedResponse, Box<dyn std::error::Error>> {
    let url = "https://api.spotify.com/v1/me/player/recently-played";
    
    println!("Fetching recently played tracks from: {}", url);
    println!("Access token length: {}", access_token.len());
    println!("Client token length: {}", client_token.len());
    
    let mut easy = Easy::new();
    let mut response_data = Vec::new();
    let mut headers = curl::easy::List::new();
    
    // Set headers
    headers.append(&format!("Authorization: Bearer {}", access_token))?;
    headers.append(&format!("client-token: {}", client_token))?;
    headers.append("Accept: */*")?;
    headers.append("User-Agent: Spoty/1.0")?;
    headers.append("Content-Type: application/json")?;
    
    println!("Headers configured, making request...");
    
    easy.url(url)?;
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
        match serde_json::from_str::<RecentlyPlayedResponse>(&response_text) {
            Ok(recently_played) => {
                println!("Successfully parsed {} recently played items", recently_played.items.len());
                Ok(recently_played)
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
            403 => "Forbidden - insufficient permissions or invalid client token.".to_string(),
            429 => "Rate limit exceeded - too many requests. Please wait a moment and try again.".to_string(),
            500..=599 => "Spotify API server error - please try again later.".to_string(),
            _ => format!("API request failed with status: {} - {}", status_code, response_text)
        };
        
        Err(error_msg.into())
    }
}