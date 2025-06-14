use gtk4::prelude::*;
use gtk4::Application;
use std::sync::{Arc, Mutex};
use crate::spotify::auth::load_auth_config;
use tokio::sync::mpsc;

mod login_window;
mod main_window;

use login_window::LoginWindow;
use main_window::MainWindow;

#[derive(Clone)]
pub struct AppState {
    pub app: Application,
    pub current_window: Arc<Mutex<Option<gtk4::ApplicationWindow>>>,
    pub access_token: Arc<Mutex<Option<String>>>,
    pub shutdown_sender: Option<mpsc::Sender<()>>,
}

pub async fn launch_gui(shutdown_sender: Option<mpsc::Sender<()>>) {
    let app = Application::builder()
        .application_id("com.carisma.spoty")
        .build();

    // Set dark theme preference globally
    app.connect_startup(|_| {
        if let Some(settings) = gtk4::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(true);
        }
    });

    let app_state = AppState {
        app: app.clone(),
        current_window: Arc::new(Mutex::new(None)),
        access_token: Arc::new(Mutex::new(None)),
        shutdown_sender,
    };

    let (auth_tx, auth_rx) = mpsc::channel::<String>(1);
    crate::handlers::set_auth_complete_sender(auth_tx);

    let auth_rx = Arc::new(Mutex::new(Some(auth_rx)));

    app.connect_activate(move |_app| {
        let state = app_state.clone();
        
        // Check if user is already authorized
        if let Some(token) = load_auth_config() {
            // Store token and show main window directly
            *state.access_token.lock().unwrap() = Some(token);
            show_main_window(state);
        } else {
            // Show login window and wait for authentication
            show_login_window(state.clone());
            
            // Spawn task to wait for authentication completion
            if let Some(mut auth_rx) = auth_rx.lock().unwrap().take() {
                glib::spawn_future_local(async move {
                    if let Some(token) = auth_rx.recv().await {
                        *state.access_token.lock().unwrap() = Some(token);
                        show_main_window(state);
                    }
                });
            }
        }
    });
    // Run the GTK application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);
}

fn show_login_window(app_state: AppState) {
    let login_window = LoginWindow::new(app_state.clone());
    login_window.show();
    
    // Store the current window
    *app_state.current_window.lock().unwrap() = Some(login_window.window().clone());
}

pub fn show_main_window(app_state: AppState) {
    // Schedule server shutdown after 5 seconds
    if let Some(sender) = app_state.shutdown_sender.clone() {
        std::thread::spawn(move || {
            println!("Main GUI opened - shutting down server in 5 seconds...");
            std::thread::sleep(std::time::Duration::from_secs(5));
            let _ = sender.blocking_send(());
        });
    }

    // Close the current window (login window)
    if let Some(window) = app_state.current_window.lock().unwrap().take() {
        window.close();
    }
    
    // Show the main window
    let main_window = MainWindow::new(app_state.clone());
    main_window.show();
    
    // Store the new current window
    *app_state.current_window.lock().unwrap() = Some(main_window.window().clone());
}