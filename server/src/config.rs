use crate::PROJECT_DIRS;
use async_std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::default::Default;
use toml;

// === SERVER CONFIG ===

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub port: Option<u16>,
    pub gh_token: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: "https://example.com".into(),
            port: Some(6969),
            gh_token: "<your github token>".into(),
        }
    }
}

pub fn get_server_config() -> ServerConfig {
    let config_dir = PROJECT_DIRS.config_dir();
    let config_path = config_dir.join("pie-server.toml");

    dbg!(&config_path);

    let config_file = std::fs::read_to_string(&config_path);

    let config: ServerConfig = match config_file {
        Ok(file) => toml::from_str(&file).unwrap(),
        Err(_) => create_default_server_config(config_path.to_str().unwrap())
            .unwrap_or(ServerConfig::default()),
    };

    dbg!(&config);

    return config;
}

fn create_default_server_config(path: &str) -> Result<ServerConfig, std::io::Error> {
    let default_config = ServerConfig::default();
    let config_str = toml::to_string_pretty(&default_config).unwrap();

    match std::fs::write(path, &config_str) {
        Ok(_) => Ok(default_config),
        Err(err) => Err(err),
    }
}

// === REPO CONFIG ===

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoConfigFile {
    pub _type: Option<String>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
    pub start_command: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RepoConfig {
    pub _type: Option<RepoConfigTypes>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
    pub start_command: String,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            _type: None,
            install_command: None,
            build_command: None,
            start_command: "".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RepoConfigTypes {
    NodeJS,
}

#[derive(Debug)]
pub enum RepoConfigError {
    InvalidTOML,
    MissingCommands,
}

pub async fn get_repo_config(path: PathBuf) -> Result<RepoConfig, RepoConfigError> {
    let mut p = path.clone();
    p.push("pie.toml");

    let config_file_res: Result<Option<RepoConfig>, RepoConfigError> =
        match async_std::fs::read_to_string(p).await {
            Ok(s) => match toml::from_str(&s) {
                Ok(s) => Ok(s),
                Err(_) => Err(RepoConfigError::InvalidTOML),
            },
            Err(_) => Ok(None),
        };
    if config_file_res.is_err() {
        return Err(RepoConfigError::InvalidTOML);
    };
    let config_file = config_file_res.ok().unwrap().unwrap_or_default();

    let default_config_res = get_default_repo_config(path.clone()).await;
    if default_config_res.is_err() {
        return default_config_res;
    }
    let default_config = default_config_res.ok().unwrap();

    Ok(RepoConfig {
        _type: value_or_def(config_file._type, default_config._type),
        install_command: value_or_def(config_file.install_command, default_config.install_command),
        build_command: value_or_def(config_file.build_command, default_config.build_command),
        start_command: if config_file.start_command != "".to_string() {
            config_file.start_command
        } else {
            default_config.start_command
        },
    })
}

async fn get_default_repo_config(path: PathBuf) -> Result<RepoConfig, RepoConfigError> {
    let mut pkg_json = path.clone();
    pkg_json.push("package.json");

    if pkg_json.is_file().await {
        return Ok(RepoConfig {
            install_command: Some("npm install".into()),
            build_command: Some("npm run build".into()),
            start_command: "npm run start".into(),
            _type: Some(RepoConfigTypes::NodeJS),
        });
    };

    Err(RepoConfigError::MissingCommands)
}

fn value_or_def<T>(value: Option<T>, def: Option<T>) -> Option<T> {
    if value.is_some() {
        return value;
    }
    def
}
