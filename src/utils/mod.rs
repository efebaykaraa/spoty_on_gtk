use rand::{distributions::Alphanumeric, Rng};

pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub mod config;
pub mod settings;
pub mod template_engine;
pub mod query_builder;
