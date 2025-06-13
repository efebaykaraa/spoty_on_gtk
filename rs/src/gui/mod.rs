use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, Box, Orientation};

pub async fn launch_gui() {
    let app = Application::builder()
        .application_id("com.carisma.spoty")
        .build();

    app.connect_activate(build_ui);
    
    // Run the GTK application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Spoty - Spotify Client")
        .default_width(400)
        .default_height(300)
        .build();

    let vbox = Box::new(Orientation::Vertical, 12);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);

    let login_button = Button::with_label("Login to Spotify");
    
    login_button.connect_clicked(|_| {
        println!("Login button clicked!");
        
        // Open the login URL in the default browser
        if let Err(e) = open::that("http://127.0.0.1:8888/login") {
            eprintln!("Failed to open browser: {}", e);
        }
    });

    vbox.append(&login_button);
    window.set_child(Some(&vbox));
    window.present();
}
