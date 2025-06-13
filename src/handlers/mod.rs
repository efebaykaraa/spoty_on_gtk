use actix_web::{web, HttpResponse, Result, rt};
use crate::spotify::auth::{CallbackQuery, get_auth_url, exchange_code_for_token};
use crate::utils::generate_random_string;
use crate::templates::MessageTemplate;

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
        return serve_template(template, false); // Don't shutdown on error
    }

    let code = match &query.code {
        Some(code) => code,
        None => {
            println!("✗ No authorization code received");
            let template = MessageTemplate::no_code_error();
            return serve_template(template, false); // Don't shutdown on error
        }
    };

    // Exchange code for access token using the auth module
    match exchange_code_for_token(code).await {
        Ok(_token_response) => {
            println!("✓ Successfully authenticated with Spotify");
            
            // Launch GTK application
            launch_gtk_app();
            
            let template = MessageTemplate::success();
            serve_template(template, true) // Shutdown on success
        }
        Err(e) => {
            println!("✗ Token exchange failed: {}", e);
            let template = MessageTemplate::token_exchange_error(&format!("Token exchange failed: {}", e));
            serve_template(template, false) // Don't shutdown on error
        }
    }
}

fn serve_template(template: MessageTemplate, should_shutdown: bool) -> Result<HttpResponse> {
    match template.render() {
        Ok(html_content) => {
            if should_shutdown {
                // Schedule shutdown after a 5 second delay to ensure response is sent
                actix_web::rt::spawn(async {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    rt::System::current().stop();
                });
            }
            
            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(html_content))
        },
        Err(_) => Ok(HttpResponse::InternalServerError()
            .body("Template rendering failed"))
    }
}

fn launch_gtk_app() {
    std::thread::spawn(|| {
        // Try different GTK applications in order of preference
        let gtk_commands = [
            ("gtk3-demo", Vec::<&str>::new()),
            ("gtk4-demo", Vec::<&str>::new()),
            ("gnome-calculator", Vec::<&str>::new()),
            ("gedit", Vec::<&str>::new()),
            ("nautilus", vec!["--new-window"]),
        ];
        
        let mut launched = false;
        for (cmd, args) in gtk_commands.iter() {
            if let Ok(_) = std::process::Command::new(cmd)
                .args(args)
                .spawn()
            {
                launched = true;
                break;
            }
        }
        
        if !launched {
            // Try a simple notification instead
            let _ = std::process::Command::new("notify-send")
                .args(&["Spoty", "Spotify OAuth successful!"])
                .spawn();
        }
    });
}
