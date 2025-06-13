use actix_web::{App, HttpServer};
use dotenv::dotenv;
use std::env;

mod auth;
mod utils;
mod handlers;
mod gui;
mod template_engine;
mod templates;

use handlers::{login, callback};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1:8888".to_string());
    
    println!("Launching Actix server on http://{}", host);
    
    // Launch GUI in a separate thread
    let gui_handle = tokio::spawn(async {
        gui::launch_gui().await;
    });

    // Start HTTP server
    let server_handle = tokio::spawn(async move {
        HttpServer::new(|| {
            App::new()
                .route("/login", actix_web::web::get().to(login))
                .route("/callback", actix_web::web::get().to(callback))
        })
        .bind(&host)
        .expect("Failed to bind server")
        .run()
        .await
        .expect("Server failed to run");
    });

    // Wait for both tasks
    tokio::try_join!(gui_handle, server_handle).unwrap();
    
    Ok(())
}