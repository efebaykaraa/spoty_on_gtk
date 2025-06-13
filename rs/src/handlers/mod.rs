use actix_web::{web, HttpResponse, Result};
use std::env;
use std::fs;
use crate::auth::{CallbackQuery, TokenRequest};
use crate::utils::generate_random_string;

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
        let html_content = fs::read_to_string("templates/error.html")
            .unwrap_or_else(|_| format!(
                r#"<html><body><h1>Authorization Failed</h1><p>Error: {}</p><p><strong>You can now close this window and return to the app.</strong></p></body></html>"#,
                error
            ));
        return Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(html_content.replace("{{error}}", error)));
    }

    let code = match &query.code {
        Some(code) => {
            println!("Received authorization code: {}", code);
            code
        },
        None => {
            println!("No authorization code received");
            let html_content = fs::read_to_string("templates/no_code.html")
                .unwrap_or_else(|_| r#"<html><body><h1>Authorization Failed</h1><p>No authorization code received.</p><p><strong>You can now close this window and return to the app.</strong></p></body></html>"#.to_string());
            return Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(html_content));
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
            let html_content = fs::read_to_string("templates/success.html")
                .unwrap_or_else(|_| r#"<html><body><h1>Authorization Successful!</h1><p>You have successfully authorized the application.</p><p><strong>You can now close this window and return to the app.</strong></p></body></html>"#.to_string());
            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(html_content))
        }
        Err(e) => {
            println!("Token exchange failed: {}", e);
            let html_content = fs::read_to_string("templates/token_error.html")
                .unwrap_or_else(|_| format!(
                    r#"<html><body><h1>Token Exchange Failed</h1><p>Error: {}</p><p><strong>You can now close this window and return to the app.</strong></p></body></html>"#,
                    e
                ));
            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(html_content.replace("{{error}}", &e.to_string())))
        }
    }
}
