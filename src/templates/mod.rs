use std::collections::HashMap;
use crate::template_engine;

pub struct MessageTemplate {
    pub title: String,
    pub message: String,
    pub hue: i16,
}

impl MessageTemplate {
    pub fn success() -> Self {
        Self {
            title: "Authorization Successful!".to_string(),
            message: "You have successfully authorized the Spoty application to access your Spotify account.".to_string(),
            hue: 210, // Green hue for success
        }
    }
    
    pub fn authorization_error(error_msg: &str) -> Self {
        Self {
            title: "Authorization Failed".to_string(),
            message: format!("Error: {}", error_msg),
            hue: 100, // Orange hue for error
        }
    }
    
    pub fn no_code_error() -> Self {
        Self {
            title: "Authorization Failed".to_string(),
            message: "No authorization code received from Spotify.".to_string(),
            hue: 100, // Orange hue for error
        }
    }
    
    pub fn token_exchange_error(error_msg: &str) -> Self {
        Self {
            title: "Token Exchange Failed".to_string(),
            message: format!("Failed to exchange authorization code for access token: {}", error_msg),
            hue: 100, // Orange hue for error
        }
    }
    
    pub fn to_variables(&self) -> HashMap<String, String> {
        let mut variables = HashMap::new();
        variables.insert("hue".to_string(), self.hue.to_string());
        variables.insert("title".to_string(), self.title.clone());
        variables.insert("message".to_string(), self.message.clone());
        variables
    }
    
    pub fn render(&self) -> Result<String, std::io::Error> {
        template_engine::render_template("message.html", self.to_variables())
    }
}
