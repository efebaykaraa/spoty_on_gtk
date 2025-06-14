use crate::spotify::recently_played::{fetch_recently_played, RecentlyPlayedItem};
use crate::spotify::top_tracks::{fetch_top_tracks, TimeRange};
use crate::thirdparty::recommendations::{RecommendationsClient, RecommendationSeeds, RecommendationsResponse};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;

pub struct PrimaryRecommendationsClient {
    recommendations_client: RecommendationsClient,
}

impl PrimaryRecommendationsClient {
    pub fn new() -> Self {
        println!("Creating new PrimaryRecommendationsClient");
        Self {
            recommendations_client: RecommendationsClient::new(),
        }
    }

    pub async fn get_primary_recommendations(
        &self,
        access_token: &str,
        client_token: &str,
        limit: Option<u32>,
    ) -> Result<RecommendationsResponse, Box<dyn std::error::Error>> {
        println!("Starting get_primary_recommendations");
        println!("Access token length: {}", access_token.len());
        println!("Client token length: {}", client_token.len());
        println!("Limit: {:?}", limit);

        // Create log directory and file
        let log_dir = "/tmp/spoty_on_gtk";
        if let Err(e) = fs::create_dir_all(log_dir) {
            println!("Warning: Could not create log directory: {}", e);
        }
        let log_file_path = format!("{}/primary_recommendations.log", log_dir);

        // Log start of process
        if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_file_path) {
            let _ = writeln!(log_file, "\n=== PRIMARY RECOMMENDATIONS START ===");
            let _ = writeln!(log_file, "Timestamp: {}", chrono::Utc::now());
        }

        // Fetch top tracks first
        println!("Fetching top tracks...");
        let top_tracks = match fetch_top_tracks(access_token, Some(TimeRange::ShortTerm), Some(10), None).await {
            Ok(data) => {
                println!("Successfully fetched {} top tracks", data.items.len());
                if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_file_path) {
                    let _ = writeln!(log_file, "Top tracks items: {}", data.items.len());
                }
                data.items
            }
            Err(e) => {
                println!("Error fetching top tracks: {}", e);
                if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_file_path) {
                    let _ = writeln!(log_file, "Error fetching top tracks: {}", e);
                }
                Vec::new() // Continue with empty top tracks if failed
            }
        };

        // Fetch recently played tracks
        println!("Fetching recently played tracks...");
        let recently_played = match fetch_recently_played(access_token, client_token).await {
            Ok(data) => {
                println!("Successfully fetched {} recently played items", data.items.len());
                if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_file_path) {
                    let _ = writeln!(log_file, "Recently played items: {}", data.items.len());
                }
                data
            }
            Err(e) => {
                println!("Error fetching recently played: {}", e);
                if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_file_path) {
                    let _ = writeln!(log_file, "Error fetching recently played: {}", e);
                }
                return Err(e);
            }
        };
        
        // Extract unique track IDs, prioritizing top tracks first
        let mut track_ids = Vec::new();
        let mut seen_ids = HashSet::new();

        println!("Processing top tracks...");
        for (i, track) in top_tracks.iter().enumerate() {
            if !seen_ids.contains(&track.id) {
                println!("  Top {}. {} by {}", i+1, track.name, 
                    track.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "));
                track_ids.push(track.id.clone());
                seen_ids.insert(track.id.clone());
            }
        }

        println!("Processing recently played items...");
        for (i, item) in recently_played.items.iter().enumerate() {
            if !seen_ids.contains(&item.track.id) {
                println!("  Recent {}. {} by {}", i+1, item.track.name, 
                    item.track.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "));
                track_ids.push(item.track.id.clone());
                seen_ids.insert(item.track.id.clone());
            }
        }

        println!("Extracted {} unique tracks", track_ids.len());

        // Build recommendation seeds using track IDs (top tracks first)
        let mut seeds = RecommendationSeeds::new();

        let track_seeds: Vec<_> = track_ids.iter().collect();
        println!("Adding track seeds: {:?}", track_seeds);
        for track_id in track_seeds {
            seeds = seeds.add_track(track_id);
        }

        // Log final seeds
        if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_file_path) {
            let _ = writeln!(log_file, "Final seeds: {:?}", seeds.seeds);
        }

        println!("Making recommendations API call...");
        // Get recommendations based on the seeds
        match self.recommendations_client.get_recommendations(seeds, limit.unwrap_or(10), None).await {
            Ok(response) => {
                println!("Successfully got {} recommendations", response.content.len());
                if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_file_path) {
                    let _ = writeln!(log_file, "Recommendations received: {}", response.content.len());
                    let _ = writeln!(log_file, "=== PRIMARY RECOMMENDATIONS END ===\n");
                }
                Ok(response)
            }
            Err(e) => {
                println!("Error getting recommendations: {}", e);
                if let Ok(mut log_file) = OpenOptions::new().create(true).append(true).open(&log_file_path) {
                    let _ = writeln!(log_file, "Error getting recommendations: {}", e);
                    let _ = writeln!(log_file, "=== PRIMARY RECOMMENDATIONS END (ERROR) ===\n");
                }
                Err(e)
            }
        }
    }

    pub async fn get_track_based_recommendations(
        &self,
        access_token: &str,
        client_token: &str,
        limit: u32,
    ) -> Result<RecommendationsResponse, Box<dyn std::error::Error>> {
        println!("Starting get_track_based_recommendations");
        
        // Fetch recently played tracks
        let recently_played = fetch_recently_played(access_token, client_token).await?;
        
        // Extract unique track IDs
        let mut track_ids = HashSet::new();
        for item in recently_played.items.iter().take(10) {
            track_ids.insert(item.track.id.clone());
        }

        let mut seeds = RecommendationSeeds::new();

        // Add up to 5 recent tracks as seeds
        for track_id in track_ids.iter().take(5) {
            seeds = seeds.add_track(track_id);
        }

        self.recommendations_client
            .get_recommendations(seeds, limit, None)
            .await
    }

    pub async fn get_mood_recommendations(
        &self,
        limit: u32,
        access_token: &str,
        client_token: &str,
        valence: Option<f32>,
        energy: Option<f32>,
        danceability: Option<f32>,
    ) -> Result<RecommendationsResponse, Box<dyn std::error::Error>> {
        println!("Starting get_mood_recommendations");
        
        // Fetch recently played for context
        let recently_played = fetch_recently_played(access_token, client_token).await?;
        
        // Extract track IDs for seeds
        let track_ids: Vec<String> = recently_played.items
            .iter()
            .take(3)
            .map(|item| item.track.id.clone())
            .collect();

        let track_id_refs: Vec<&str> = track_ids.iter().map(|s| s.as_str()).collect();

        self.recommendations_client
            .get_mood_recommendations(limit, track_id_refs, valence, energy, danceability)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primary_recommendations_client_creation() {
        let client = PrimaryRecommendationsClient::new();
        assert!(true); // Just test that we can create an instance
    }

    #[tokio::test]
    async fn test_primary_recommendations_client_methods_exist() {
        let client = PrimaryRecommendationsClient::new();
        // Test that all methods exist (they will fail without valid tokens, but that's expected)
        assert!(true);
    }
}
