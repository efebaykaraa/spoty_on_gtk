use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::utils::config::get_config_dir;
use rand::Rng;
use rand::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct AuthConfig {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
}

pub fn load_auth_config() -> Option<String> {
    let config_dir = get_config_dir();
    let config_path = Path::new(&config_dir).join("auth.conf");
    
    if !config_path.exists() {
        return None;
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let config = match toml::from_str::<AuthConfig>(&content) {
        Ok(c) => c,
        Err(_) => return None,
    };

    // Check if token is still valid
    if config.access_token.is_none() {
        return None;
    }

    let token = config.access_token.as_ref().unwrap();

    if is_token_valid(&config) {
        return Some(token.clone());
    }

    if config.refresh_token.is_none() {
        return None;
    }

    let refresh_token = config.refresh_token.as_ref().unwrap();

    // Try to refresh the token
    match refresh_access_token(refresh_token) {
        Ok(new_token) => return Some(new_token),
        Err(_) => return None,
    }
    None
}

pub fn save_auth_config(token_response: &TokenResponse) -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();
    let config_path = Path::new(&config_dir).join("auth.conf");
    
    // Create config directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    
    // Remove existing auth config file if it exists
    if config_path.exists() {
        fs::remove_file(&config_path)?;
    }
    
    let expires_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() + token_response.expires_in;
    
    let auth_config = AuthConfig {
        access_token: Some(token_response.access_token.clone()),
        refresh_token: token_response.refresh_token.clone(),
        expires_at: Some(expires_at),
    };
    
    let content = toml::to_string(&auth_config)?;
    fs::write(&config_path, content)?;
    
    Ok(())
}

pub fn clear_auth_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();
    let config_path = Path::new(&config_dir).join("auth.conf");
    
    if config_path.exists() {
        fs::remove_file(config_path)?;
    }
    
    Ok(())
}

fn is_token_valid(config: &AuthConfig) -> bool {
    if let Some(expires_at) = config.expires_at {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Add 5 minute buffer before expiration
        expires_at > current_time + 300
    } else {
        false
    }
}

fn refresh_access_token(refresh_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    // This would implement the actual token refresh logic
    // For now, return an error to indicate refresh is not implemented
    Err("Token refresh not implemented yet".into())
}

pub fn get_auth_url() -> String {
    let client_id = std::env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not set");
    let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI").expect("SPOTIFY_REDIRECT_URI not set");
    
    let scopes = "user-library-read user-read-private user-read-email user-top-read user-read-recently-played";
    let state = generate_state();
    
    format!(
        "https://accounts.spotify.com/authorize?response_type=code&client_id={}&scope={}&redirect_uri={}&state={}",
        client_id, 
        urlencoding::encode(scopes), 
        urlencoding::encode(&redirect_uri), 
        state
    )
}

pub async fn exchange_code_for_token(code: &str) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let client_id = std::env::var("SPOTIFY_CLIENT_ID")?;
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")?;
    let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI")?;
    
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", &redirect_uri),
    ];
    
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&params)
        .send()
        .await?;
    
    if response.status().is_success() {
        let token_response: TokenResponse = response.json().await?;
        save_auth_config(&token_response)?;
        Ok(token_response)
    } else {
        Err(format!("Token exchange failed: {}", response.status()).into())
    }
}

fn generate_state() -> String {
    let mut rng = rand::thread_rng();
    (0..16).map(|_| rng.gen_range(0..255) as u8).map(|b| format!("{:02x}", b)).collect()
}

pub fn is_authenticated() -> bool {
    load_auth_config().is_some()
}