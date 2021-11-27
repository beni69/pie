use crate::{git, runner, CONFIG, PROJECT_DIRS};
use async_std::{
    fs,
    path::{Path, PathBuf},
};
use pie_lib::utils::{get_unix_time, split_repo};
use serde::{Deserialize, Serialize};
use serde_json::json;
use surf::{Client, Error, StatusCode, Url};

lazy_static! {
    static ref CLIENT: Client = surf::Config::new()
        .set_base_url(Url::parse("https://api.github.com").unwrap())
        .add_header("Accept", "application/vnd.github.v3+json")
        .unwrap()
        .add_header("Authorization", format!("token {}", CONFIG.gh_token))
        .unwrap()
        .try_into()
        .unwrap();
}

#[derive(Debug)]
pub enum GitHubError {
    NotFound,
    AccessDenied,
    Http(Error),
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubHookBody {
    repository: GitHubRepository,
}
#[derive(Serialize, Deserialize, Debug)]
struct GitHubRepository {
    full_name: String,
    hooks_url: String,
}

async fn get_repo(repo_name: &str) -> Result<GitHubRepository, GitHubError> {
    let r = split_repo(repo_name);

    let res = CLIENT
        .get(format!("/repos/{owner}/{repo}", owner = r.0, repo = r.1))
        .send()
        .await;

    if res.is_err() {
        return Err(GitHubError::Http(res.unwrap_err()));
    }
    let mut res = res.unwrap();

    if res.status() == StatusCode::NotFound {
        return Err(GitHubError::NotFound);
    }

    let data = res.body_json::<GitHubRepository>().await;

    if data.is_err() {
        return Err(GitHubError::Http(data.unwrap_err()));
    }
    let data = data.unwrap();

    Ok(data)
}

pub async fn init_repo(repo_name: &str) -> Result<(), GitHubError> {
    let r = split_repo(repo_name);

    let repo = get_repo(repo_name).await;
    if repo.is_err() {
        return Err(repo.unwrap_err());
    }
    let _repo = repo?;

    let j = json!({"name": "web", "config": {"url": String::from(&CONFIG.url) + "/handler", "content_type": "json"}});

    let res = CLIENT
        .post(format!(
            "/repos/{owner}/{repo}/hooks",
            owner = r.0,
            repo = r.1
        ))
        .body(j)
        .send()
        .await;

    match res {
        Ok(r) => {
            if r.status() == StatusCode::NotFound {
                Err(GitHubError::AccessDenied)
            } else {
                Ok(())
            }
        }
        Err(e) => {
            dbg!(&e);
            Err(GitHubError::Http(e))
        }
    }
}

pub async fn webhook_handler(mut req: tide::Request<()>) -> tide::Result {
    let req_body = req.body_string().await?;
    let hook_event = req.header("X-GitHub-Event").unwrap().as_str();

    // write hook body to a file
    let mut hooks_folder = PathBuf::from(PROJECT_DIRS.data_local_dir());
    hooks_folder.push("hooks");
    if !hooks_folder.is_dir().await {
        fs::create_dir_all(&hooks_folder).await?;
    }
    let hook_id = String::from(get_unix_time().to_string());
    fs::write(
        Path::join(
            &hooks_folder.as_path(),
            format!("{date}-{event}.json", date = hook_id, event = hook_event),
        ),
        &req_body,
    )
    .await?;

    return match hook_event {
        "ping" => Ok("pong".into()),
        "push" => {
            println!("push");

            let body: GitHubHookBody = match serde_json::from_str(&req_body) {
                Ok(b) => b,
                Err(e) => panic!("{}", e),
            };

            git::pull(&body.repository.full_name).await.unwrap();

            runner::run(&body.repository.full_name).await.unwrap();

            return Ok("pull successful".into());
        }

        _ => Ok(tide::Response::builder(404).body("event not found").build()),
    };
}
