use crate::{CONFIG, PROJECT_DIRS};
use async_std::{
    fs,
    path::{Path, PathBuf},
};
use git2::{build::RepoBuilder, Cred, Error, ErrorCode, FetchOptions, RemoteCallbacks, Repository};

pub enum GitCloneError {
    Exists,
    NotFound,
}

pub async fn clone(repo: &str, force: bool) -> Result<(), GitCloneError> {
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
        return Err(GitCloneError::Exists);
    }

    let mut builder = RepoBuilder::new();
    let mut callbacks = RemoteCallbacks::new();
    let mut fetch_opts = FetchOptions::new();

    callbacks.credentials(|_, user, _| {
        let creds = Cred::userpass_plaintext(user.unwrap_or(&CONFIG.gh_token), &CONFIG.gh_token);

        Ok(creds.unwrap())
    });
    fetch_opts.remote_callbacks(callbacks);
    builder.fetch_options(fetch_opts);

    return match builder.clone(&repo, std::path::PathBuf::from(&dirname).as_path()) {
        Ok(_) => Ok(()),
        Err(_) => Err(GitCloneError::NotFound),
    };
}

pub async fn pull(repo: &str) -> Result<(), Error> {
    let r = (repo.split('/')).collect::<Vec<&str>>();
    let mut repo_dir = PathBuf::from(PROJECT_DIRS.data_local_dir());
    repo_dir.push("repos");
    repo_dir.push(r[1]);
    let repo = Repository::open(&repo_dir)?;

    dbg!(repo.path());

    let branch = &get_current_branch_name(repo_dir).unwrap_or("master".to_string());
    dbg!(branch);

    let mut callbacks = RemoteCallbacks::new();
    let mut fetch_opts = FetchOptions::new();

    callbacks.credentials(|_, user, _| {
        let creds = Cred::userpass_plaintext(user.unwrap_or(&CONFIG.gh_token), &CONFIG.gh_token);

        Ok(creds.unwrap())
    });
    fetch_opts.remote_callbacks(callbacks);

    repo.find_remote("origin")?
        .fetch(&[branch], Some(&mut fetch_opts), None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let analysis = repo.merge_analysis(&[&fetch_commit])?;
    if analysis.0.is_up_to_date() {
        println!("up to date");
        return Ok(());
    } else if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch);
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        repo.set_head(&refname)?;
        return repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()));
    } else {
        return Err(Error::from_str("Fast-forward only!"));
    }
}

fn get_current_branch_name(repo_dir: PathBuf) -> Result<String, Error> {
    let repo = Repository::open(repo_dir)?;
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => return Err(e),
    };
    let head = head.as_ref().and_then(|h| h.shorthand());
    let branch = head.unwrap_or("master");
    Ok(branch.to_owned())
}
