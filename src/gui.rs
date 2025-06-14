use slint::ComponentHandle;

slint::slint!{
    export { AppWindow } from "ui/app.slint";
}

pub async fn launch_gui() {
    let ui = AppWindow::new().unwrap();
    
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
            // Update status after successful browser launch
            ui.set_status_text("Waiting for Spotify authentication...".into());
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
