use gtk4::{glib, prelude::*};
use gtk4::{ApplicationWindow, Label, Box, Orientation, ScrolledWindow, ListBox};
use crate::gui::AppState;
use crate::spotify::primary_recommendations::PrimaryRecommendationsClient;
use crate::thirdparty::recommendations::RecommendedTrack;

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
        let welcome_label = Label::new(Some("Recommended for You"));
        welcome_label.add_css_class("title-1");

        // Scrolled window for content
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled_window.set_vexpand(true);

        let content_list = ListBox::new();
        content_list.add_css_class("boxed-list");
        scrolled_window.set_child(Some(&content_list));

        vbox.append(&welcome_label);
        vbox.append(&scrolled_window);

        // Load and display primary recommendations
        let content_list_clone = content_list.clone();
        let access_token = app_state.access_token.lock().unwrap().clone();
        glib::spawn_future_local(async move {
            Self::load_primary_recommendations(content_list_clone, access_token).await;
        });

        window.set_child(Some(&vbox));

        Self { window }
    }

    async fn load_primary_recommendations(content_list: ListBox, access_token: Option<String>) {
        let access_token = match access_token {
            Some(token) => token,
            None => {
                let error_label = Label::new(Some("Please login to view your recommendations"));
                error_label.add_css_class("dim-label");
                content_list.append(&error_label);
                return;
            }
        };

        let client_token = ""; // TODO: Get client_token from somewhere
        let recommendations_client = PrimaryRecommendationsClient::new();
        
        match recommendations_client.get_primary_recommendations(&access_token, client_token, Some(20)).await {
            Ok(response) => {
                let track_count = response.content.len();
                for track in response.content {
                    let track_row = Self::create_recommendation_row(&track);
                    content_list.append(&track_row);
                }
                
                let status_text = if track_count == 0 {
                    "No recommendations found".to_string()
                } else {
                    format!("{} recommendations loaded", track_count)
                };
                
                println!("{}", status_text);
            }
            Err(e) => {
                let error_label = Label::new(Some(&format!("✗ Error loading recommendations: {}", e)));
                error_label.add_css_class("error");
                content_list.append(&error_label);
            }
        }
    }

    fn create_recommendation_row(track: &RecommendedTrack) -> Box {
        let row = Box::new(Orientation::Horizontal, 12);
        row.set_margin_top(8);
        row.set_margin_bottom(8);
        row.set_margin_start(12);
        row.set_margin_end(12);

        // Track info
        let info_box = Box::new(Orientation::Vertical, 4);
        
        let title_label = Label::new(Some(&track.track_title));
        title_label.set_halign(gtk4::Align::Start);
        title_label.add_css_class("heading");
        
        let artist_names: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
        let artists_text = artist_names.join(", ");
        let artist_label = Label::new(Some(&artists_text));
        artist_label.set_halign(gtk4::Align::Start);
        artist_label.add_css_class("dim-label");
        
        let duration_minutes = track.duration_ms / 60000;
        let duration_seconds = (track.duration_ms % 60000) / 1000;
        let details = format!("{}:{:02} • Popularity: {}", duration_minutes, duration_seconds, track.popularity);
        let details_label = Label::new(Some(&details));
        details_label.set_halign(gtk4::Align::Start);
        details_label.add_css_class("caption");

        let id_label = Label::new(Some(&format!("ID: {}", track.id)));
        id_label.set_halign(gtk4::Align::Start);
        id_label.add_css_class("caption");

        info_box.append(&title_label);
        info_box.append(&artist_label);
        info_box.append(&details_label);
        info_box.append(&id_label);

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
