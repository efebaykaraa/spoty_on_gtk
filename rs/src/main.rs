use actix_web::{App, HttpServer};
use dotenv::dotenv;
use std::env;

mod auth;
mod utils;
mod handlers;
mod gui;
mod svelte;

use handlers::{login, callback};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let should_compile = args.contains(&"-s".to_string()) || args.contains(&"--svelte-compile".to_string());

    // Handle Svelte compilation
    if should_compile {
        println!("Compiling Svelte components with pnpm build...");
        svelte::compile_svelte_components().await?;
        println!("Svelte compilation completed.");
    } else if !svelte::check_compiled_files_exist() {
        println!("Compiled Svelte files not found, creating fallback templates...");
        svelte::compile_svelte_components().await?;
        println!("Fallback templates created.");
    }

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1:8888".to_string());
    
    println!("Server running on http://{}", host);
    
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