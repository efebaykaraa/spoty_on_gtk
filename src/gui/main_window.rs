use gtk4::{glib, prelude::*};
use gtk4::{ApplicationWindow, Label, Box, Orientation, ScrolledWindow, ListBox};
use crate::gui::AppState;
use crate::utils::settings::load_settings;
use crate::spotify::albums::{fetch_top_items, TopItem, Track};

pub struct MainWindow {
    window: ApplicationWindow,
}

impl MainWindow {
    pub fn new(app_state: AppState) -> Self {
        let window = ApplicationWindow::builder()
            .application(&app_state.app)
            .title("Spoty - Spotify Client")
            .default_width(800)
            .default_height(600)
            .build();

        let vbox = Box::new(Orientation::Vertical, 12);
        vbox.set_margin_top(20);
        vbox.set_margin_bottom(20);
        vbox.set_margin_start(20);
        vbox.set_margin_end(20);

        // Title
        let welcome_label = Label::new(Some("Your Top Tracks"));
        welcome_label.add_css_class("title-1");

        // Scrolled window for tracks
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled_window.set_vexpand(true);

        let tracks_list = ListBox::new();
        tracks_list.add_css_class("boxed-list");
        scrolled_window.set_child(Some(&tracks_list));

        vbox.append(&welcome_label);
        vbox.append(&scrolled_window);

        // Load and display tracks - properly pass the access token from AppState
        let tracks_list_clone = tracks_list.clone();
        let access_token = app_state.access_token.lock().unwrap().clone();
        glib::spawn_future_local(async move {
            Self::load_tracks(tracks_list_clone, access_token).await;
        });

        window.set_child(Some(&vbox));

        Self { window }
    }

    async fn load_tracks(tracks_list: ListBox, access_token: Option<String>) {
        let settings = load_settings();
        
        let access_token = match access_token {
            Some(token) => token,
            None => {
                let error_label = Label::new(Some("Please login to view your top tracks"));
                error_label.add_css_class("dim-label");
                tracks_list.append(&error_label);
                return;
            }
        };

        match fetch_top_items(&settings, &access_token, "tracks", &settings.time_range, 0).await {
            Ok(response) => {
                let mut track_count = 0;
                for item in response.items {
                    if let TopItem::Track(track) = item {
                        let track_row = Self::create_track_row(&track);
                        tracks_list.append(&track_row);
                        track_count += 1;
                    }
                }
                
                // Add status information
                let status_text = if track_count == 0 {
                    "No tracks found".to_string()
                } else {
                    format!("✓ Successfully fetched {} track{}", track_count, if track_count == 1 { "" } else { "s" })
                };
                
                let status_label = Label::new(Some(&status_text));
                status_label.add_css_class(if track_count == 0 { "dim-label" } else { "success" });
                status_label.set_margin_top(12);
                tracks_list.append(&status_label);
            }
            Err(e) => {
                let error_label = Label::new(Some(&format!("✗ Error loading tracks: {}", e)));
                error_label.add_css_class("error");
                tracks_list.append(&error_label);
            }
        }
    }

    fn create_track_row(track: &Track) -> Box {
        let row = Box::new(Orientation::Horizontal, 12);
        row.set_margin_top(8);
        row.set_margin_bottom(8);
        row.set_margin_start(12);
        row.set_margin_end(12);

        // Track info
        let info_box = Box::new(Orientation::Vertical, 4);
        
        let title_label = Label::new(Some(&track.name));
        title_label.set_halign(gtk4::Align::Start);
        title_label.add_css_class("heading");
        
        let artist_names: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
        let artists_text = artist_names.join(", ");
        let artist_label = Label::new(Some(&artists_text));
        artist_label.set_halign(gtk4::Align::Start);
        artist_label.add_css_class("dim-label");
        
        let album_label = Label::new(Some(&track.album.name));
        album_label.set_halign(gtk4::Align::Start);
        album_label.add_css_class("caption");

        let duration_minutes = track.duration_ms / 60000;
        let duration_seconds = (track.duration_ms % 60000) / 1000;
        let details = format!("{}:{:02} • Popularity: {}", duration_minutes, duration_seconds, track.popularity);
        let details_label = Label::new(Some(&details));
        details_label.set_halign(gtk4::Align::Start);
        details_label.add_css_class("caption");

        info_box.append(&title_label);
        info_box.append(&artist_label);
        info_box.append(&album_label);
        info_box.append(&details_label);

        row.append(&info_box);
        row
    }

    pub fn show(&self) {
        self.window.present();
    }

    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }
}
