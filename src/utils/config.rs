use std::env;

pub fn get_config_dir() -> String {
    env::var("CONFIG_PATH")
        .unwrap_or_else(|_| {
            let home = env::var("HOME").expect("HOME environment variable not set");
            format!("{}/.config/spoty_on_qt", home)
        })
}
