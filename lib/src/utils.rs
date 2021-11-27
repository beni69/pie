use crate::PROJECT_DIRS;
use async_std::path::PathBuf;
use rand;
use std::{
    io::Error,
    process::{Command, Stdio},
    str::FromStr,
    time::SystemTime,
};
use url::Url;

pub fn get_unix_time() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(t) => t.as_millis(),
        Err(_) => 0,
    }
}

pub fn repo_to_url(repo: &str) -> String {
    format!("https://github.com/{}", repo)
}

pub fn url_to_repo(url: &str) -> Option<String> {
    let u = match Url::from_str(&url) {
        Ok(x) => x,
        Err(_) => return None,
    };
    debug!("{:?}", &u);

    let path = u.path_segments()?.collect::<Vec<&str>>();

    Some(path[0].to_owned() + "/" + path[1])
}

pub fn split_repo(repo: &str) -> (&str, &str) {
    let v = repo.split('/').collect::<Vec<&str>>();
    (v[0], v[1])
}

pub fn repo_to_path(repo: &str) -> PathBuf {
    let r = split_repo(repo);

    let mut d = PathBuf::from(PROJECT_DIRS.data_local_dir());
    d.push("repos");
    d.push(r.0);
    d.push(r.1);

    d
}

pub fn repo_to_pie_name(repo: &str) -> String {
    let r = split_repo(repo);

    format!("pie-{}-{}", r.0, r.1)
}

pub fn string_to_cmd_and_args(s: &str) -> (&str, Vec<&str>) {
    let v = s.split_ascii_whitespace().collect::<Vec<&str>>();
    let first = v.split_first().unwrap_or((&"", &[]));
    (*first.0, first.1.to_vec())
}

pub fn exec_sync(cmd: &str, dir: std::path::PathBuf) -> Result<String, Error> {
    let cmd_args = string_to_cmd_and_args(cmd);

    let cmd = Command::new(cmd_args.0)
        .args(cmd_args.1)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let out = cmd.wait_with_output()?;

    Ok(String::from_utf8(out.stdout).unwrap_or(String::new()))
}

pub fn generate_key() -> String {
    let alphabet =
        String::from("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNPQRSTUVWXYZ0123456789/.;!@$");

    let mut code = String::new();

    for _ in 0..16 {
        let number = rand::random::<f32>() * (alphabet.len() as f32);
        let number = number.round() as usize;

        match alphabet.chars().nth(number) {
            Some(c) => code.push(c),
            None => {}
        }
    }
    debug!("{:?}", &code);
    code
}
