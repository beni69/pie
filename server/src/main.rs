#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
use crate::{git::GitError, github::GitHubError};
use directories::ProjectDirs;
use pie_lib::{
    config::{get_server_config, RepoConfigError, ServerConfig},
    utils::{create_logs_dir, string_to_cmd_and_args, url_to_repo},
};
use runner::RunnerError;
use tide::{
    prelude::{Deserialize, Serialize},
    Redirect, Request, Response, Result,
};
mod git;
mod github;
mod runner;

//* GET /ping
async fn ping(_req: Request<()>) -> Result {
    Ok("pong!".into())
}

//* POST /handler
// this endpoint is for github webhooks, and not the user
async fn handler(req: Request<()>) -> Result {
    if req.header("X-GitHub-Event").is_none() {
        return Ok(Response::builder(418)
            .body("this endpoint is reserved for GitHub webhooks")
            .build());
    }

    github::webhook_handler(req).await
}

//* POST /deploy
#[derive(Debug, Deserialize, Serialize)]
pub struct DeployParams {
    repo: String,
    force: Option<bool>,
}
async fn deploy(mut req: Request<()>) -> Result {
    let params: DeployParams = req.body_json().await?;

    let repo = url_to_repo(&params.repo);
    if repo.is_none() {
        return Ok(Response::builder(400)
            .body("The `repo` parameter is not a url to a valid GitHub repository")
            .build());
    }
    let repo = repo.unwrap();

    // clone the repo
    let clone = git::clone(&repo, params.force.unwrap_or(false)).await;
    if clone.is_err() {
        return Ok(match clone.unwrap_err() {
            GitError::Exists => Response::builder(400).body("Error while cloning: already exists. Run with `force: true` to force re-reploy!").build(),
            GitError::NotFound => Response::builder(404).body("Error while cloning: repository not found!").build(),
        });
    };

    // create github webhook
    let gh = github::init_repo(&repo).await;
    if gh.is_err() {
        return Ok(match gh.unwrap_err() {
            GitHubError::NotFound => Response::builder(500)
                .body("unable to reach the repo from the github api.\nat this point, cloning the repo was successful, this error should never occour")
                .build(),
            GitHubError::AccessDenied=>Response::builder(400).body("You don't have write access to the git repository. This error usually occours when you deploy a public repo you don't own.").build(),
            GitHubError::Http(err) => {
                error!("github request failed with unknown error: {:?}", &err);
                Response::builder(500)
                    .body("unknown error! check the server logs")
                    .build()
            },
        });
    };

    // run the code
    let run = runner::run(&repo).await;
    if run.is_err() {
        return Ok(match run.unwrap_err() {
            RunnerError::CommandFailed => Response::builder(400)
                .body("Error while running commands!")
                .build(),
            RunnerError::RepoConfigError(e) => match e{
                RepoConfigError::InvalidTOML => Response::builder(400).body("`pie.toml` is not a valid TOML file.").build(),
                RepoConfigError::MissingCommands => Response::builder(400).body("Your project type could not be auto-detected, and your pie.toml doesn't exist, or doesn't have a start command.").build(),
            },
        });
    }

    Ok(format!("Successfully cloned {}", &params.repo).into())
}

//* POST /exec
// execute a command. (for testing purposes)
async fn exec(mut req: Request<()>) -> Result {
    let cmd = req.body_string().await?;
    let c = string_to_cmd_and_args(&cmd);
    let res = runner::exec(c.0, c.1, async_std::path::PathBuf::from("/tmp")).await;
    match res {
        Ok(s) => Ok(s.into()),
        Err(e) => Ok(e.to_string().into()),
    }
}

lazy_static! {
    static ref CONFIG: ServerConfig = get_server_config();
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "beni69", "pie").unwrap();
}

#[async_std::main]
async fn main() -> Result<()> {
    // setup logger
    pretty_env_logger::init();
    info!("pie server starting up!");

    // server directories setup
    create_logs_dir().expect("unable to create logs dir");

    // setup http server
    let mut app = tide::new();
    app.with(driftwood::DevLogger);
    app.at("/")
        .get(Redirect::new("https://github.com/beni69/pie"));
    app.at("/ping").get(ping);
    app.at("/handler").all(handler);
    app.at("/deploy").post(deploy);
    app.at("/exec").post(exec);
    let host = format!("127.0.0.1:{}", &CONFIG.port.unwrap());
    app.listen(&host).await?;

    Ok(())
}
