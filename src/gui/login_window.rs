use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Button, Box, Orientation};
use crate::gui::{AppState, show_main_window};

pub struct LoginWindow {
    window: ApplicationWindow,
}

impl LoginWindow {
    pub fn new(app_state: AppState) -> Self {
        let window = ApplicationWindow::builder()
            .application(&app_state.app)
            .title("Spoty - Login")
            .default_width(400)
            .default_height(300)
            .build();

        let vbox = Box::new(Orientation::Vertical, 12);
        vbox.set_margin_top(20);
        vbox.set_margin_bottom(20);
        vbox.set_margin_start(20);
        vbox.set_margin_end(20);

        let login_button = Button::with_label("Login to Spotify");
        
        login_button.connect_clicked(move |_| {
            // Open the login URL in the default browser
            if let Err(e) = open::that("http://127.0.0.1:8888/login") {
                eprintln!("Failed to open browser: {}", e);
            }
        });

        vbox.append(&login_button);
        window.set_child(Some(&vbox));

        Self { window }
    }

    pub fn show(&self) {
        self.window.present();
    }

    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }
}
