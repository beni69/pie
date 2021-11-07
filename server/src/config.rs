use crate::PROJECT_DIRS;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub url: String,
    pub port: Option<u16>,
    pub gh_token: String,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            url: "https://example.com".into(),
            port: Some(6969),
            gh_token: "<your github token>".into(),
        }
    }
}

pub fn get_config() -> Config {
    let config_dir = PROJECT_DIRS.config_dir();
    let config_path = config_dir.join("pie-server.toml");

    dbg!(&config_path);

    let config_file = fs::read_to_string(&config_path);

    let config: Config = match config_file {
        Ok(file) => toml::from_str(&file).unwrap(),
        Err(_) => create_default_config(config_path.to_str().unwrap()).unwrap_or(Config::default()),
    };

    dbg!(&config);

    return config;
}

fn create_default_config(path: &str) -> Result<Config, std::io::Error> {
    let default_config = Config::default();
    let config_str = toml::to_string_pretty(&default_config).unwrap();

    match fs::write(path, &config_str) {
        Ok(_) => Ok(default_config),
        Err(err) => Err(err),
    }
}
