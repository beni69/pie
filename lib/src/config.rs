use crate::PROJECT_DIRS;
use async_std::{fs::read_to_string, path::PathBuf};
use serde::{Deserialize, Serialize};
use serde_json;
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

    info!("Using server config file at {:?}", &config_path);

    let config_file = std::fs::read_to_string(&config_path);

    let config: ServerConfig = match config_file {
        Ok(file) => toml::from_str(&file).unwrap(),
        Err(_) => create_default_server_config(config_path.to_str().unwrap())
            .unwrap_or(ServerConfig::default()),
    };

    debug!("{:?}", &config);

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

#[derive(Debug)]
pub struct RepoConfigFile {
    pub _type: Option<String>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
    pub start_command: Option<String>,
}
#[derive(Debug, Deserialize)]
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
    NodeTS,
}

#[derive(Debug)]
pub enum RepoConfigError {
    InvalidTOML,
    MissingCommands,
}

pub async fn get_repo_config(path: PathBuf) -> Result<RepoConfig, RepoConfigError> {
    let mut p = path.clone();
    p.push("pie.toml");

    let config_file_res = match read_to_string(&p).await {
        Ok(s) => match toml::from_str::<RepoConfig>(&s) {
            // TODO: read file into RepoConfigFile and serialize it into RepoConfig
            Ok(x) => Ok(Some(x)),
            Err(_) => Err(RepoConfigError::InvalidTOML),
        },
        Err(_) => Ok(None),
    };

    if config_file_res.is_err() {
        error!("invalid config file: {:?}", &p);
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
        install_command: value_or_def_null(
            config_file.install_command,
            default_config.install_command,
        ),
        build_command: value_or_def_null(config_file.build_command, default_config.build_command),
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

    //== NODE PROJECT ==//
    if pkg_json.is_file().await {
        let j = parse_pkg_json(pkg_json.clone()).await;
        let yarn = use_yarn(path.clone()).await;
        let install_command = npm_yarn_run("install", yarn);

        if j.is_some() {
            let j = j.unwrap();
            let build_command = if j.scripts.pie_build.is_some() {
                Some(npm_yarn_run("run pie-build", yarn))
            } else {
                Some(npm_yarn_run("run build", yarn))
            };
            let start_command = if j.scripts.pie_start.is_some() {
                npm_yarn_run("run pie-start", yarn)
            } else {
                if j.scripts.start.is_some() {
                    npm_yarn_run("run start", yarn)
                } else {
                    "node .".into()
                }
            };

            return Ok(RepoConfig {
                _type: Some(RepoConfigTypes::NodeJS),
                install_command: Some(install_command),
                build_command: build_command,
                start_command: start_command,
            });
        } else {
            // no sensible defaults found, returning to the primitive way
            return Ok(RepoConfig {
                _type: Some(RepoConfigTypes::NodeJS),
                install_command: Some("npm install".into()),
                build_command: None,
                start_command: "npm run start".into(),
            });
        }
    };

    Err(RepoConfigError::MissingCommands)
}

#[derive(Debug, Deserialize)]
struct PackageJSON {
    scripts: PackageJSONScripts,
}
#[derive(Debug, Deserialize)]
struct PackageJSONScripts {
    build: Option<String>,
    start: Option<String>,
    pie_build: Option<String>,
    pie_start: Option<String>,
}
async fn parse_pkg_json(path: PathBuf) -> Option<PackageJSON> {
    let file = read_to_string(path).await;
    if file.is_err() {
        return None;
    }
    let j = serde_json::from_str::<PackageJSON>(&file.unwrap());

    match j {
        Ok(j) => Some(j),
        Err(_) => None,
    }
}

async fn use_yarn(path: PathBuf) -> bool {
    let mut p = path.clone();
    p.push("yarn.lock");
    p.is_file().await
}
fn npm_yarn_run(cmd: &str, yarn: bool) -> String {
    if yarn {
        format!("yarn {}", cmd)
    } else {
        format!("npm {}", cmd)
    }
}

fn value_or_def<T>(value: Option<T>, def: Option<T>) -> Option<T> {
    if value.is_some() {
        return value;
    }
    def
}
fn value_or_def_null(value: Option<String>, def: Option<String>) -> Option<String> {
    if value == Some("NONE".into()) {
        debug!("value {:?} is NONE", value);
        return None;
    }
    if value.is_some() {
        return value;
    }
    def
}
