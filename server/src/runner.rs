use crate::{
    config::get_repo_config,
    utils::{repo_to_path, string_to_cmd_and_args},
};
use async_std::{
    path::PathBuf,
    process::{Command, Stdio},
};
use std::{io::Error, result::Result};

#[derive(Debug)]
pub enum RunnerError {
    CommandsNotFound,
    CommandFailed,
}

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

async fn run_repo_cmd(cmd: &str, p: PathBuf) -> Result<String, RunnerError> {
    let cmd_args = string_to_cmd_and_args(cmd);
    let cmd_res = exec(cmd_args.0, cmd_args.1, p).await;

    if cmd_res.is_ok() {
        println!("{}", cmd_res.as_ref().unwrap());
    }

    match cmd_res {
        Ok(x) => Ok(x),
        Err(_) => Err(RunnerError::CommandFailed),
    }
}

pub async fn run(repo: &str) -> Result<(), RunnerError> {
    let repo_path = repo_to_path(repo);
    let repo_config_res = get_repo_config(repo_path.clone()).await;

    if repo_config_res.is_none() {
        return Err(RunnerError::CommandsNotFound);
    };

    let repo_config = repo_config_res.unwrap();

    run_repo_cmd(&repo_config.install_command.unwrap(), repo_path.clone()).await?;
    run_repo_cmd(&repo_config.build_command.unwrap(), repo_path.clone()).await?;
    run_repo_cmd(&repo_config.start_command, repo_path.clone()).await?;

    Ok(())
}