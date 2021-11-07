use crate::PROJECT_DIRS;
use async_std::path::PathBuf;
use reqwest::Url;
use std::{str::FromStr, time::SystemTime};

pub fn get_unix_time() -> u128 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(t) => t.as_millis(),
        Err(_) => 0,
    }
}

pub fn repo_to_url(repo: &str) -> String {
    format!("https://github.com/{}", repo)
}

pub fn _url_to_repo(url: &str) -> String {
    let u = Url::from_str(&url).unwrap();
    dbg!(&u);

    let path = u.path_segments().unwrap().collect::<Vec<&str>>();

    path[0].to_owned() + "/" + path[1]
}

pub fn _split_repo(repo: &str) -> (&str, &str) {
    let v = repo.split('/').collect::<Vec<&str>>();
    (v[0], v[1])
}

pub fn repo_to_path(repo: &str) -> PathBuf {
    let r = _split_repo(repo);

    let mut d = PathBuf::from(PROJECT_DIRS.data_local_dir());
    d.push("repos");
    d.push(r.0);
    d.push(r.1);

    d
}
