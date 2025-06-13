use gtk4::{glib, prelude::*};
use gtk4::{ApplicationWindow, Label, Box, Orientation, ScrolledWindow, ListBox};
use crate::gui::AppState;
use crate::utils::settings::{Settings, load_settings};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct SpotifyAlbumsResponse {
    items: Vec<AlbumItem>,
    total: u32,
    offset: u32,
    limit: u32,
}

#[derive(Deserialize)]
struct AlbumItem {
    added_at: String,
    album: Album,
}

#[derive(Deserialize)]
struct Album {
    name: String,
    artists: Vec<Artist>,
    images: Vec<AlbumImage>,
    release_date: String,
    total_tracks: u32,
}

#[derive(Deserialize)]
struct Artist {
    name: String,
}

#[derive(Deserialize)]
struct AlbumImage {
    url: String,
    height: u32,
    width: u32,
}

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
        let welcome_label = Label::new(Some("Your Albums"));
        welcome_label.add_css_class("title-1");

        // Scrolled window for albums
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled_window.set_vexpand(true);

        let albums_list = ListBox::new();
        albums_list.add_css_class("boxed-list");
        scrolled_window.set_child(Some(&albums_list));

        vbox.append(&welcome_label);
        vbox.append(&scrolled_window);

        // Load and display albums
        let albums_list_clone = albums_list.clone();
        glib::spawn_future_local(async move {
            Self::load_albums(albums_list_clone).await;
        });

        window.set_child(Some(&vbox));

        Self { window }
    }

    async fn load_albums(albums_list: ListBox) {
        let settings = load_settings();
        
        // TODO: Get access token from OAuth flow
        let access_token = ""; // This should come from your OAuth implementation
        
        if access_token.is_empty() {
            let error_label = Label::new(Some("Please login to view your albums"));
            error_label.add_css_class("dim-label");
            albums_list.append(&error_label);
            return;
        }

        match Self::fetch_albums(&settings, access_token, 0).await {
            Ok(response) => {
                for item in response.items {
                    let album_row = Self::create_album_row(&item.album);
                    albums_list.append(&album_row);
                }
            }
            Err(e) => {
                let error_label = Label::new(Some(&format!("Error loading albums: {:?}", e)));
                error_label.add_css_class("error");
                albums_list.append(&error_label);
            }
        }
    }

    async fn fetch_albums(settings: &Settings, access_token: &str, offset: u32) -> Result<SpotifyAlbumsResponse, std::boxed::Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.spotify.com/v1/me/albums?limit={}&offset={}&market={}",
            settings.limit, offset, settings.market
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await?;

        if response.status().is_success() {
            let albums: SpotifyAlbumsResponse = response.json().await?;
            Ok(albums)
        } else {
            Err(format!("API request failed: {}", response.status()).into())
        }
    }

    fn create_album_row(album: &Album) -> Box {
        let row = Box::new(Orientation::Horizontal, 12);
        row.set_margin_top(8);
        row.set_margin_bottom(8);
        row.set_margin_start(12);
        row.set_margin_end(12);

        // Album info
        let info_box = Box::new(Orientation::Vertical, 4);
        
        let title_label = Label::new(Some(&album.name));
        title_label.set_halign(gtk4::Align::Start);
        title_label.add_css_class("heading");
        
        let artist_names: Vec<&str> = album.artists.iter().map(|a| a.name.as_str()).collect();
        let artists_text = artist_names.join(", ");
        let artist_label = Label::new(Some(&artists_text));
        artist_label.set_halign(gtk4::Align::Start);
        artist_label.add_css_class("dim-label");
        
        let details = format!("{} • {} tracks", album.release_date, album.total_tracks);
        let details_label = Label::new(Some(&details));
        details_label.set_halign(gtk4::Align::Start);
        details_label.add_css_class("caption");

        info_box.append(&title_label);
        info_box.append(&artist_label);
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
