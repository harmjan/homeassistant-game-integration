use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::darksouls;

/// The application config
#[derive(Deserialize)]
pub struct Config {
    pub debug_window: bool,
    pub dark_souls: Option<darksouls::Config>,
}

/// Load the application config from the given file
pub fn load_config<S>(filename: S) -> Config
where
    S: AsRef<Path>,
{
    let config_file = fs::read_to_string(filename).expect("Failed to read config file");
    toml::from_str(&config_file).expect("Failed to parse config file")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Make sure that loading the supplied example file works
    #[test]
    fn test_load_example_config() {
        let config_file =
            std::fs::read_to_string("config.toml.example").expect("Failed to read config file");
        let result = toml::from_str::<Config>(&config_file);
        assert!(result.is_ok());
    }
}
