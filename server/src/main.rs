#[macro_use]
extern crate lazy_static;
use crate::{config::RepoConfigError, git::GitCloneError};
use directories::ProjectDirs;
use runner::RunnerError;
use tide::{
    prelude::{Deserialize, Serialize},
    Redirect, Request, Response, Result,
};
mod config;
mod git;
mod github;
mod runner;
mod utils;

// GET /ping
async fn ping(_req: Request<()>) -> Result {
    Ok("pong!".into())
}

// POST /handler
// this endpoint is for github webhooks, and not the user
async fn handler(req: Request<()>) -> Result {
    if req.header("X-GitHub-Event").is_none() {
        return Ok(Response::builder(400)
            .body("this endpoint is reserved for GitHub webhooks")
            .build());
    }

    github::webhook_handler(req).await
}

// POST /deploy
#[derive(Debug, Deserialize, Serialize)]
pub struct DeployParams {
    repo: String,
    force: Option<bool>,
}
async fn deploy(mut req: Request<()>) -> Result {
    let params: DeployParams = req.body_json().await?;

    match git::clone(&params.repo, params.force.unwrap_or(false)).await {
        Ok(_) => {
            github::init_repo(&params.repo).await?;

            match runner::run(&params.repo).await {
                Ok(_) => Ok(format!("Successfully cloned {}", &params.repo).into()),
                Err(e) => Ok(match e {
                    RunnerError::CommandFailed => Response::builder(400).body("Error while running commands!").build(),
                    RunnerError::RepoConfigError(e) => match e {
                        RepoConfigError::InvalidTOML => Response::builder(400).body("pie.toml is not a valid TOML file.").build(),
                        RepoConfigError::MissingCommands => Response::builder(400).body("Your project type could not be auto-detected, and your pie.toml doesn't exist, or doesn't have a start command.").build(),
                    },

                }),

            }
        }
        Err(e) => {
           return Ok(match e{
            GitCloneError::Exists => Response::builder(400).body("Error while cloning: already exists. Run with `force: true` to force re-reploy!").build(),
            GitCloneError::NotFound => Response::builder(404).body("Error while cloning: repository not found!").build(),
        })
        }
    }
}

lazy_static! {
    static ref CONFIG: config::ServerConfig = config::get_server_config();
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "beni69", "pie").unwrap();
}

#[async_std::main]
async fn main() -> Result<()> {
    let mut app = tide::new();
    app.with(driftwood::DevLogger);
    app.at("/")
        .get(Redirect::new("https://github.com/beni69/pie"));
    app.at("/ping").get(ping);
    app.at("/handler").all(handler);
    app.at("/deploy").post(deploy);
    let host = format!("127.0.0.1:{}", &CONFIG.port.unwrap());
    println!("listening on: http://{}", host);
    app.listen(&host).await?;

    Ok(())
}
