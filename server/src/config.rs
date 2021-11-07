use crate::PROJECT_DIRS;
use async_std::path::PathBuf;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub port: Option<u16>,
    pub gh_token: String,
}

impl std::default::Default for ServerConfig {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoConfig {
    pub _type: Option<String>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
    pub start_command: String,
}

pub async fn get_repo_config(path: PathBuf) -> Option<RepoConfig> {
    let mut p = path.clone();
    p.push("pie.toml");

    match async_std::fs::read_to_string(p).await {
        Ok(s) => match toml::from_str(&s) {
            Ok(s) => s,
            Err(_) => None,
        },
        Err(_) => get_default_repo_config(path.clone()).await,
    }
}

async fn get_default_repo_config(path: PathBuf) -> Option<RepoConfig> {
    let mut pkg_json = path.clone();
    pkg_json.push("package.json");

    if pkg_json.is_file().await {
        return Some(RepoConfig {
            install_command: Some("npm install".into()),
            build_command: Some("npm run build".into()),
            start_command: "npm run start".into(),
            _type: None,
        });
    };

    None
}
