use async_std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use std::io::Error;

use crate::utils::repo_to_path;

async fn exec(cmd: &str, args: Vec<&str>, dir: PathBuf) -> Result<String, Error> {
    let cmd = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let out = cmd.output().await?;

    Ok(String::from_utf8(out.stdout).unwrap_or(String::new()))
}

// very basic demo using yarn
// TODO: run command based on configs
pub async fn run(repo: &str) -> Result<(), Error> {
    let repo_path = repo_to_path(repo);

    let cmd_install = exec(
        "yarn",
        ["--no-progress", "--non-interactive", "install"].to_vec(),
        repo_path.clone(),
    )
    .await?;
    dbg!(cmd_install);
    let cmd_build = exec(
        "yarn",
        ["--no-progress", "--non-interactive", "run", "build"].to_vec(),
        repo_path.clone(),
    )
    .await?;
    dbg!(cmd_build);
    let cmd_start = exec(
        "yarn",
        ["--no-progress", "--non-interactive", "run", "start"].to_vec(),
        repo_path.clone(),
    )
    .await?;
    dbg!(cmd_start);

    Ok(())
}
