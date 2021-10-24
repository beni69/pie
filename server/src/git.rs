use crate::PROJECT_DIRS;
use async_std::{fs, path::Path};
use git2::Repository;

pub async fn clone(repo: &str, force: bool) -> bool {
    // get directory name to clone to from url
    let mut dirname: String = String::from(repo);
    if dirname.ends_with(".git") {
        dirname = dirname.strip_suffix(".git").unwrap().to_string();
    };
    dirname = PROJECT_DIRS.data_local_dir().to_str().unwrap().to_string()
        + "/repos/"
        + (dirname.split('/').collect::<Vec<&str>>()).last().unwrap();

    // directory exists
    let dir_exists = Path::new(&dirname).exists().await;
    if dir_exists && force {
        match fs::remove_dir_all(&dirname).await {
            Err(e) => panic!("{}", e),
            Ok(x) => x,
        };
    } else if dir_exists {
        return false;
    }

    match Repository::clone(&repo, &dirname) {
        Ok(repo) => repo,
        Err(e) => panic!("falied to clone: {:?}", e),
    };

    return true;
}
