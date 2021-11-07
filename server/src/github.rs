use crate::utils::get_unix_time;
use crate::{git, runner, CONFIG, PROJECT_DIRS};
use async_std::{
    fs,
    path::{Path, PathBuf},
};
use octocrab::Octocrab;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

lazy_static! {
    static ref CRAB: Octocrab = Octocrab::builder()
        .personal_token(String::from(&CONFIG.gh_token))
        .build()
        .unwrap();
    static ref CLIENT: Client = reqwest::Client::builder()
        .user_agent("pie/0.1.0-alpha.1")
        .build()
        .unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubHookBody {
    // repository: Repository,
    repository: GitHubHookRepo,
}
#[derive(Serialize, Deserialize, Debug)]
struct GitHubHookRepo {
    full_name: String,
}

pub async fn init_repo(repo_name: &str) -> reqwest::Result<()> {
    let r = (repo_name.split('/')).collect::<Vec<&str>>();

    let repo = CRAB.repos(r[0], r[1]).get().await.unwrap();

    let j = json!({"name": "web", "config": {"url": String::from(&CONFIG.url) + "/handler", "content_type": "json"}});

    let _res = CLIENT
        .post(repo.hooks_url.as_str())
        .header("accept", "application/vnd.github.v3+json")
        .header("Authorization", format!("token {}", CONFIG.gh_token))
        .header("content-type", "application/json")
        .body(j.to_string())
        .send()
        .await?;

    Ok(())
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
