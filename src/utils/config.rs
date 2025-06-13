use std::env;

pub fn get_config_dir() -> String {
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "~/.config/spoty_on_qt".to_string());
    if config_path.starts_with("~/") {
        let home = env::var("HOME").expect("HOME environment variable not set");
        config_path.replace("~", &home)
    } else {
        config_path
    }
}
