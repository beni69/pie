#[macro_use]
extern crate lazy_static;
use directories::ProjectDirs;
use tide::{
    prelude::{Deserialize, Serialize},
    Request, Response, Result,
};
mod config;
mod git;
mod github;
use git::GitCloneError;

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
    let repo_url = format!("https://github.com/{}", &params.repo);

    match git::clone(&repo_url, params.force.unwrap_or(false)).await {
        Ok(_) => {
            github::init_repo(&params.repo).await?;

            return Ok(format!("Successfully cloned {}", &params.repo).into());
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
    static ref CONFIG: config::Config = config::get_config();
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "beni69", "pie").unwrap();
}

#[async_std::main]
async fn main() -> Result<()> {
    dbg!(&CONFIG.url);

    let mut app = tide::new();
    app.with(driftwood::DevLogger);
    app.at("/").get(ping);
    app.at("/ping").get(ping);
    app.at("/handler").all(handler);
    app.at("/deploy").post(deploy);
    let host = format!("127.0.0.1:{}", &CONFIG.port.unwrap());
    println!("listening on: http://{}", host);
    app.listen(&host).await?;
    Ok(())
}
