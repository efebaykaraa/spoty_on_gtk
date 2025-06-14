use actix_web::{web, HttpResponse, Result};
use crate::spotify::auth::{CallbackQuery, get_auth_url, exchange_code_for_token};
use crate::templates::MessageTemplate;
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};

pub async fn login() -> Result<HttpResponse> {
    let auth_url = get_auth_url();

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish())
}

pub async fn callback(query: web::Query<CallbackQuery>) -> Result<HttpResponse> {
    if let Some(error) = &query.error {
        println!("✗ OAuth authorization failed: {}", error);
        let template = MessageTemplate::authorization_error(error);
        return serve_template(template);
    }

    let code = match &query.code {
        Some(code) => code,
        None => {
            println!("✗ No authorization code received");
            let template = MessageTemplate::no_code_error();
            return serve_template(template);
        }
    };

    // Exchange code for access token using the auth module
    match exchange_code_for_token(code).await {
        Ok(token_response) => {
            println!("✓ Successfully authenticated with Spotify");
            
            // After successful token exchange, notify GUI
            if let Some(sender) = AUTH_COMPLETE_SENDER.lock().unwrap().as_ref() {
                let _ = sender.send(token_response.access_token.clone()).await;
            }
            
            let template = MessageTemplate::success();
            serve_template(template)
        }
        Err(e) => {
            println!("✗ Token exchange failed: {}", e);
            let template = MessageTemplate::token_exchange_error(&format!("Token exchange failed: {}", e));
            serve_template(template)
        }
    }
}

fn serve_template(template: MessageTemplate) -> Result<HttpResponse> {
    match template.render() {
        Ok(html_content) => {
            // Note: Server shutdown is now handled by the GUI after 5 seconds
            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(html_content))
        },
        Err(_) => Ok(HttpResponse::InternalServerError()
            .body("Template rendering failed"))
    }
}

// Global sender for authentication completion
lazy_static::lazy_static! {
    static ref AUTH_COMPLETE_SENDER: Arc<Mutex<Option<mpsc::Sender<String>>>> = Arc::new(Mutex::new(None));
}

pub fn set_auth_complete_sender(sender: mpsc::Sender<String>) {
    *AUTH_COMPLETE_SENDER.lock().unwrap() = Some(sender);
}
