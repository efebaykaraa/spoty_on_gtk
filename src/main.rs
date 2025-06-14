use actix_web::{App, HttpServer};
use dotenv::dotenv;
use std::env;
use tokio::sync::mpsc;
use clap::{Arg, Command};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

mod thirdparty;
mod spotify;
mod utils;
mod handlers;
mod gui;
mod template_engine;
mod templates;
mod debug;

use handlers::{login, callback};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    // Parse command line arguments
    let matches = Command::new("Spoty")
        .version("1.0")
        .about("Spotify Desktop Client")
        .arg(
            Arg::new("delete-config")
                .short('d')
                .long("delete-config")
                .help("Delete the existing configuration folder")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();
    
    // Handle delete config flag
    if matches.get_flag("delete-config") {
        if let Err(e) = debug::delete_config() {
            eprintln!("Error deleting configuration: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }
    
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1:8888".to_string());
    
    println!("Starting Spoty server on http://{}", host);
    
    // Create a flag to track if callback has been called
    let callback_called = Arc::new(AtomicBool::new(false));
    let callback_called_clone = callback_called.clone();

    // Create a channel for shutdown signal
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    
    // Launch GUI in a separate thread, passing the shutdown sender
    let gui_handle = tokio::spawn(async move {
        gui::launch_gui(Some(shutdown_tx)).await;
    });

    // Start HTTP server
    let server_handle = tokio::spawn(async move {
        let server = HttpServer::new(move || {
            let callback_flag = callback_called_clone.clone();
            App::new()
                .app_data(actix_web::web::Data::new(callback_flag))
                .route("/login", actix_web::web::get().to(login))
                .route("/callback", actix_web::web::get().to(move |query: actix_web::web::Query<spotify::auth::CallbackQuery>, data: actix_web::web::Data<Arc<AtomicBool>>| {
                    let flag = data.get_ref().clone();
                    async move {
                        let result = callback(query).await;
                        // Set flag that callback was called
                        flag.store(true, Ordering::Relaxed);
                        result
                    }
                }))
        })
        .bind(&host)
        .expect("Failed to bind server")
        .run();
        
        // Keep server running until shutdown signal AND callback has been called
        tokio::select! {
            result = server => {
                if let Err(e) = result {
                    eprintln!("Server error: {}", e);
                }
                println!("Server stopped");
            }
            _ = shutdown_rx.recv() => {
                println!("Shutdown signal received, stopping server...");
            }
        }
    });

    // Wait for both tasks
    tokio::try_join!(gui_handle, server_handle).unwrap();
    
    Ok(())
}