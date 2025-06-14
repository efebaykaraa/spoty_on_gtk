use serde::{Deserialize, Serialize};
use crate::utils::query_builder::QueryBuilder;
use curl::easy::Easy;
use std::io::Write;

const RECCOBEATS_RECOMMENDATIONS_URL: &str = "https://api.reccobeats.com/v1/track/recommendation";

#[derive(Debug, Clone, Serialize)]
pub struct RecommendationRequest {
    pub size: u32,
    pub seeds: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acousticness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub danceability: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrumentalness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liveness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loudness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speechiness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valence: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub popularity: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct RecommendationsResponse {
    pub content: Vec<RecommendedTrack>,
}

#[derive(Debug, Deserialize)]
pub struct RecommendedTrack {
    pub id: String,
    #[serde(rename = "trackTitle")]
    pub track_title: String,
    pub artists: Vec<Artist>,
    #[serde(rename = "durationMs")]
    pub duration_ms: u32,
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
    pub href: String,
    #[serde(rename = "availableCountries")]
    pub available_countries: String,
    pub popularity: u32,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub href: String,
}

#[derive(Debug, Clone)]
pub struct RecommendationSeeds {
    pub seeds: Vec<String>,
}

impl RecommendationSeeds {
    pub fn new() -> Self {
        Self {
            seeds: Vec::new(),
        }
    }

    pub fn add_track(mut self, track_id: &str) -> Self {
        if self.seeds.len() < 5 {
            self.seeds.push(track_id.to_string());
        }
        self
    }

    pub fn add_artist(mut self, artist_id: &str) -> Self {
        if self.seeds.len() < 5 {
            self.seeds.push(artist_id.to_string());
        }
        self
    }

    fn is_valid(&self) -> bool {
        !self.seeds.is_empty()
    }
}

pub struct RecommendationsClient {
}

impl RecommendationsClient {
    pub fn new() -> Self {
        println!("Creating new RecommendationsClient for ReccoBeats API");
        Self {}
    }

    pub async fn get_recommendations(
        &self,
        seeds: RecommendationSeeds,
        size: u32,
        audio_features: Option<AudioFeatures>,
    ) -> Result<RecommendationsResponse, Box<dyn std::error::Error>> {
        if !seeds.is_valid() {
            return Err("Invalid seeds: must have 1 seed at least".into());
        }

        let mut query_builder = QueryBuilder::new()
            .add_u32("size", size)
            .add_string_vec("seeds", seeds.seeds);

        // Only add audio features if they have values
        if let Some(features) = audio_features {
            query_builder = query_builder
                .add_optional_f32("acousticness", features.acousticness)
                .add_optional_f32("danceability", features.danceability)
                .add_optional_f32("energy", features.energy)
                .add_optional_f32("instrumentalness", features.instrumentalness)
                .add_optional_i32("key", features.key)
                .add_optional_f32("liveness", features.liveness)
                .add_optional_f32("loudness", features.loudness)
                .add_optional_i32("mode", features.mode)
                .add_optional_f32("speechiness", features.speechiness)
                .add_optional_f32("tempo", features.tempo)
                .add_optional_f32("valence", features.valence)
                .add_optional_u32("popularity", features.popularity);
        }

        let url = query_builder.build_with_url(RECCOBEATS_RECOMMENDATIONS_URL);
        println!("curl -X GET \"{}\" -H \"Accept: application/json\"", url);

        let mut easy = Easy::new();
        let mut response_data = Vec::new();
        let mut headers = curl::easy::List::new();

        headers.append("Accept: application/json")?;

        easy.url(&url)?;
        easy.http_headers(headers)?;
        easy.timeout(std::time::Duration::from_secs(10))?;
        easy.connect_timeout(std::time::Duration::from_secs(5))?;

        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                response_data.extend_from_slice(data);
                Ok(data.len())
            })?;

            if let Err(e) = transfer.perform() {
                return Err(format!("Curl error: {}", e).into());
            }
        }

        let status_code = easy.response_code()?;
        let response_text = String::from_utf8(response_data)?;

        println!("ReccoBeats API response status: {}", status_code);

        if status_code >= 200 && status_code < 300 {
            let recommendations: RecommendationsResponse = serde_json::from_str(&response_text)?;
            println!("Successfully received {} recommendations from ReccoBeats", recommendations.content.len());

            // Log some sample recommendations
            for (i, track) in recommendations.content.iter().enumerate().take(3) {
                println!("  {}. {} by {}", i+1, track.track_title, 
                    track.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", "));
            }

            Ok(recommendations)
        } else {
            println!("ReccoBeats API error: {}", response_text);
            Err(format!("ReccoBeats API error: {} - {}", status_code, response_text).into())
        }
    }

    pub async fn get_recommendations_by_tracks(
        &self,
        track_ids: Vec<&str>,
        size: u32,
    ) -> Result<RecommendationsResponse, Box<dyn std::error::Error>> {
        let mut seeds = RecommendationSeeds::new();
        for track_id in track_ids.into_iter().take(5) {
            seeds = seeds.add_track(track_id);
        }
        self.get_recommendations(seeds, size, None).await
    }

    pub async fn get_mood_recommendations(
        &self,
        size: u32,
        track_ids: Vec<&str>,
        valence: Option<f32>,
        energy: Option<f32>,
        danceability: Option<f32>,
    ) -> Result<RecommendationsResponse, Box<dyn std::error::Error>> {
        let mut seeds = RecommendationSeeds::new();
        for track_id in track_ids.into_iter().take(5) {
            seeds = seeds.add_track(track_id);
        }

        let audio_features = AudioFeatures {
            valence,
            energy,
            danceability,
            ..Default::default()
        };

        self.get_recommendations(seeds, size, Some(audio_features)).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct AudioFeatures {
    pub acousticness: Option<f32>,
    pub danceability: Option<f32>,
    pub energy: Option<f32>,
    pub instrumentalness: Option<f32>,
    pub key: Option<i32>,
    pub liveness: Option<f32>,
    pub loudness: Option<f32>,
    pub mode: Option<i32>,
    pub speechiness: Option<f32>,
    pub tempo: Option<f32>,
    pub valence: Option<f32>,
    pub popularity: Option<u32>,
}

impl AudioFeatures {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_valence(mut self, valence: f32) -> Self {
        self.valence = Some(valence.clamp(0.0, 1.0));
        self
    }

    pub fn with_energy(mut self, energy: f32) -> Self {
        self.energy = Some(energy.clamp(0.0, 1.0));
        self
    }

    pub fn with_danceability(mut self, danceability: f32) -> Self {
        self.danceability = Some(danceability.clamp(0.0, 1.0));
        self
    }

    pub fn with_acousticness(mut self, acousticness: f32) -> Self {
        self.acousticness = Some(acousticness.clamp(0.0, 1.0));
        self
    }

    pub fn with_tempo(mut self, tempo: f32) -> Self {
        self.tempo = Some(tempo.clamp(0.0, 250.0));
        self
    }

    pub fn with_popularity(mut self, popularity: u32) -> Self {
        self.popularity = Some(popularity.min(100));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_seeds() {
        let seeds = RecommendationSeeds::new()
            .add_track("4NHQUGzhtTLFvgF5SZesLK")
            .add_track("0c6xIDDpzE81m2q797ordA");

        assert_eq!(seeds.seeds.len(), 2);
        assert!(seeds.is_valid());
    }

    #[test]
    fn test_invalid_seeds() {
        let empty_seeds = RecommendationSeeds::new();
        assert!(!empty_seeds.is_valid());

        let mut too_many_seeds = RecommendationSeeds::new();
        for i in 0..6 {
            too_many_seeds = too_many_seeds.add_track(&format!("track_{}", i));
        }
        assert!(!too_many_seeds.is_valid());
    }

    #[test]
    fn test_audio_features() {
        let features = AudioFeatures::new()
            .with_valence(0.8)
            .with_energy(0.7)
            .with_danceability(0.9);

        assert_eq!(features.valence, Some(0.8));
        assert_eq!(features.energy, Some(0.7));
        assert_eq!(features.danceability, Some(0.9));
    }

    #[test]
    fn test_recommendations_client_creation() {
        let client = RecommendationsClient::new();
        assert!(true);
    }
}
