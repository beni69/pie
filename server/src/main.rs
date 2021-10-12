mod git;
use tide::{prelude::*, Request, Response, Result};

#[derive(Debug, Deserialize, Serialize)]
pub struct DeployParams {
    repo: String,
    force: Option<bool>,
}

async fn hello(req: Request<()>) -> Result {
    let name = req.param("name").unwrap_or("world");
    Ok(format!("hello, {}!", name).into())
}

// POST /deploy
async fn deploy(mut req: Request<()>) -> Result {
    let params: DeployParams = req.body_json().await?;
    let success = git::clone(&params.repo, params.force.unwrap_or(false), &HOME).await;

    if success {
        return Ok(format!("Successfully cloned {}", &params.repo).into());
    } else {
        return Ok(Response::builder(400)
            .body("Error while cloning: already exists. Run with `force: true` to force re-reploy!")
            .build());
    }
}

const PORT: &str = "8000";
static HOME: &str = "~/pie";

#[async_std::main]
async fn main() -> Result<()> {
    let mut app = tide::new();
    app.with(driftwood::DevLogger);
    app.at("/hello/:name").get(hello);
    app.at("/deploy").post(deploy);
    let host = format!("127.0.0.1:{}", PORT);
    println!("listening on: http://{}", host);
    app.listen(&host).await?;
    Ok(())
}
