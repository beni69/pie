use async_std::{path::PathBuf, process::Command};
use pie_lib::{
    config::{get_repo_config, RepoConfigError},
    utils::{repo_to_log_file, repo_to_path, string_to_cmd_and_args},
};
use std::{io::Error, result::Result};
use which::which;

#[derive(Debug)]
pub enum RunnerError {
    CommandFailed,
    RepoConfigError(RepoConfigError),
}

pub async fn exec(cmd: &str, args: Vec<&str>, dir: PathBuf) -> Result<String, Error> {
    debug!(
        "running command: '{} {}' in dir: '{}'",
        &cmd,
        &args.join(" "),
        &dir.to_string_lossy()
    );

    let cmd = Command::new(cmd).args(args).current_dir(dir).spawn()?;
    let out = cmd.output().await?;

    Ok(String::from_utf8(out.stdout).unwrap_or(String::new()))
}

async fn exec_daemon(cmd: &str, p: PathBuf, repo: &str) -> Result<String, Error> {
    dbg!();

    let log_file = repo_to_log_file(&repo);
    if !log_file.exists().await {
        async_std::fs::write(&log_file, "").await?;
    }

    let cmd_args = string_to_cmd_and_args(&cmd);
    let full_cmd = format!(
        "daemonize -a -c {dir} -o {logfile} -e {logfile} {cmd} {args}",
        dir = p.to_str().expect("invalid path"),
        logfile = log_file.to_str().expect("invalid path"),
        cmd = which(cmd_args.0)
            .expect("command not found!")
            .to_str()
            .expect("invalid path"),
        args = cmd_args.1.join(" ")
    );
    dbg!(&full_cmd);
    let full_cmd_args = string_to_cmd_and_args(&full_cmd);
    exec(full_cmd_args.0, full_cmd_args.1, p).await
}

async fn run_repo_cmd(cmd: &str, repo: &str, daemonize: bool) -> Result<String, RunnerError> {
    let p = repo_to_path(repo);
    let cmd_res = if daemonize {
        exec_daemon(cmd, p, &repo).await
    } else {
        let cmd_args = string_to_cmd_and_args(cmd);
        exec(cmd_args.0, cmd_args.1, p).await
    };

    if cmd_res.is_ok() {
        println!("{}", cmd_res.as_ref().unwrap());
    } else {
        println!(
            "command falied:\n{:?}\n{:?}",
            cmd,
            cmd_res.as_ref().err().unwrap()
        );
    }

    match cmd_res {
        Ok(x) => Ok(x),
        Err(_) => Err(RunnerError::CommandFailed),
    }
}

pub async fn run(repo: &str) -> Result<(), RunnerError> {
    let repo_path = repo_to_path(repo);
    let repo_config_res = get_repo_config(repo_path.clone()).await;

    if repo_config_res.is_err() {
        return Err(RunnerError::RepoConfigError(repo_config_res.err().unwrap()));
    };

    let repo_config = repo_config_res.ok().unwrap();
    debug!("running repo with config: {:?}", &repo_config);

    if repo_config.install_command.is_some() {
        println!("running install command");
        run_repo_cmd(&repo_config.install_command.unwrap(), &repo, false).await?;
    }
    if repo_config.build_command.is_some() {
        println!("running build command");
        run_repo_cmd(&repo_config.build_command.unwrap(), &repo, false).await?;
    }
    println!("running start command");
    run_repo_cmd(&repo_config.start_command, &repo, true).await?;

    Ok(())
}
