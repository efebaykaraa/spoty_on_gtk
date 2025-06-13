use actix_web::{web, HttpResponse, Result, rt};
use std::env;
use std::fs;
use std::path::PathBuf;
use crate::auth::{CallbackQuery, TokenRequest};
use crate::utils::generate_random_string;
use crate::templates::MessageTemplate;

pub async fn login() -> Result<HttpResponse> {
    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not found in environment");
    let redirect_uri = env::var("SPOTIFY_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8888/callback".to_string());
    
    let state = generate_random_string(16);
    let scope = "user-read-private user-read-email playlist-modify-public playlist-modify-private playlist-read-private playlist-read-collaborative";

    let auth_url = format!(
        "https://accounts.spotify.com/authorize?response_type=code&client_id={}&scope={}&redirect_uri={}&state={}",
        urlencoding::encode(&client_id),
        urlencoding::encode(scope),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&state)
    );

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish())
}

pub async fn callback(query: web::Query<CallbackQuery>) -> Result<HttpResponse> {
    // Log the callback parameters to terminal
    println!("=== Spotify OAuth Callback ===");
    println!("Code: {:?}", query.code);
    println!("State: {:?}", query.state);
    println!("Error: {:?}", query.error);
    println!("================================");

    if let Some(error) = &query.error {
        println!("Authorization failed with error: {}", error);
        let template = MessageTemplate::authorization_error(error);
        return serve_template(template);
    }

    let code = match &query.code {
        Some(code) => {
            println!("Received authorization code: {}", code);
            code
        },
        None => {
            println!("No authorization code received");
            let template = MessageTemplate::no_code_error();
            return serve_template(template);
        }
    };

    let state = query.state.as_deref().unwrap_or("");
    println!("State parameter: {}", state);

    // Exchange code for access token
    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID not found");
    let client_secret = env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET not found");
    let redirect_uri = env::var("SPOTIFY_REDIRECT_URI").expect("SPOTIFY_REDIRECT_URI not found");

    let token_request = TokenRequest {
        grant_type: "authorization_code".to_string(),
        code: code.clone(),
        redirect_uri,
        client_id: client_id.clone(),
        client_secret,
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://accounts.spotify.com/api/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&token_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            let token_data = resp.text().await.unwrap_or_else(|_| "Failed to read response".to_string());
            println!("Token exchange successful: {}", token_data);
            
            // Save auth data to config file
            if let Err(e) = save_auth_config(&token_data) {
                println!("Failed to save auth config: {}", e);
            } else {
                println!("Auth config saved successfully");
            }
            
            let template = MessageTemplate::success();
            serve_template(template)
        }
        Err(e) => {
            println!("Token exchange failed: {}", e);
            let template = MessageTemplate::token_exchange_error(&e.to_string());
            serve_template(template)
        }
    }
}

fn save_auth_config(token_data: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
    let config_dir = home_dir.join(".config").join("spoty_on_qt");
    
    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir)?;
    
    let config_file = config_dir.join("settings.conf");
    
    // Parse token data (assuming it's JSON) and create readable config content
    let parsed: serde_json::Value = serde_json::from_str(token_data)?;
    
    let mut config_content = String::from("[spotify_auth]\n");
    
    if let Some(access_token) = parsed.get("access_token").and_then(|v| v.as_str()) {
        config_content.push_str(&format!("access_token={}\n", access_token));
    }
    
    if let Some(refresh_token) = parsed.get("refresh_token").and_then(|v| v.as_str()) {
        config_content.push_str(&format!("refresh_token={}\n", refresh_token));
    }
    
    if let Some(token_type) = parsed.get("token_type").and_then(|v| v.as_str()) {
        config_content.push_str(&format!("token_type={}\n", token_type));
    }
    
    if let Some(expires_in) = parsed.get("expires_in").and_then(|v| v.as_u64()) {
        config_content.push_str(&format!("expires_in={}\n", expires_in));
    }
    
    if let Some(scope) = parsed.get("scope").and_then(|v| v.as_str()) {
        config_content.push_str(&format!("scope={}\n", scope));
    }
    
    // Add timestamp for when the token was saved
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    config_content.push_str(&format!("saved_at={}\n", timestamp));
    
    fs::write(config_file, config_content)?;
    Ok(())
}

fn serve_template(template: MessageTemplate) -> Result<HttpResponse> {
    match template.render() {
        Ok(html_content) => {
            // Schedule shutdown after a brief delay to ensure response is sent
            actix_web::rt::spawn(async {
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                rt::System::current().stop();
            });
            
            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(html_content))
        },
        Err(_) => Ok(HttpResponse::InternalServerError()
            .body("Template rendering failed"))
    }
}