use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

slint::slint!{
    export { AppWindow } from "ui/app.slint";
}

// Global channel for authentication success
static AUTH_SENDER: once_cell::sync::OnceCell<mpsc::UnboundedSender<()>> = once_cell::sync::OnceCell::new();

pub async fn launch_gui() {
    let ui = AppWindow::new().unwrap();
    
    // Create channel for authentication trigger
    let (auth_tx, mut auth_rx) = mpsc::unbounded_channel();
    AUTH_SENDER.set(auth_tx).expect("Failed to set auth sender");
    
    // Handle login button click
    let ui_weak = ui.as_weak();
    ui.on_login_clicked(move || {
        let ui = ui_weak.unwrap();
        ui.set_status_text("Opening browser for Spotify login...".into());
        
        // Open browser to login URL
        if let Err(e) = open::that("http://127.0.0.1:8888/login") {
            eprintln!("Failed to open browser: {}", e);
            ui.set_status_text("Failed to open browser. Please navigate to http://127.0.0.1:8888/login manually.".into());
        } else {
            ui.set_status_text("Waiting for Spotify authentication...".into());
        }
    });
    
    // Listen for authentication success trigger
    let ui_weak_auth = ui.as_weak();
    tokio::spawn(async move {
        while let Some(_) = auth_rx.recv().await {
            let ui_weak_clone = ui_weak_auth.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak_clone.upgrade() {
                    ui.set_status_text("Authentication successful! You are now logged in.".into());
                    // Add your page navigation logic here when you have the property
                    // ui.set_is_authenticated(true);
                }
            }).unwrap();
        }
    });
    
    // Handle exit button click
    let ui_weak = ui.as_weak();
    ui.on_exit_app(move || {
        let ui = ui_weak.unwrap();
        ui.window().hide().unwrap();
    });
    
    // Run the UI (blocking call)
    ui.run().unwrap();
}

// Function to be called from Actix server when authentication succeeds
pub fn trigger_auth_success() {
    if let Some(sender) = AUTH_SENDER.get() {
        if let Err(e) = sender.send(()) {
            eprintln!("Failed to send auth success trigger: {}", e);
        }
    } else {
        eprintln!("Auth sender not initialized");
    }
}