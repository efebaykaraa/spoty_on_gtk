use std::fs;
use crate::utils::config::get_config_dir;

pub fn delete_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_dir();
    let config_path = std::path::Path::new(&config_dir);
    if config_path.exists() {
        println!("Deleting configuration directory: {}", config_path.display());
        fs::remove_dir_all(&config_dir)?;
        println!("Configuration directory deleted successfully");
    } else {
        println!("Configuration directory does not exist: {}", std::path::Path::new(&config_dir).display());
    }
    
    Ok(())
}
