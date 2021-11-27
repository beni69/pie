use crate::{Deploy, CLI};
use pie_lib::utils::exec_sync;
use std::process::exit;
use surf::{Client, Error, Url};

lazy_static! {
    static ref CLIENT: Client = surf::Config::new()
        .set_base_url(Url::parse(&CLI.url).unwrap())
        .try_into()
        .unwrap();
}

pub async fn ping() -> Result<(), Error> {
    let mut res = CLIENT.get("/ping").await?;

    println!("{}", res.body_string().await?);

    Ok(())
}

pub async fn deploy(opts: Deploy) -> Result<(), Error> {
    println!("{:?}", opts);

    let remote_url = match exec_sync(
        "git config --get remote.origin.url",
        std::env::current_dir().unwrap(),
    ) {
        Ok(x) => x.trim().to_string(),
        Err(_) => todo!(),
    };
    dbg!(&remote_url);

    if remote_url.len() == 0 {
        eprintln!("No origin git remote found.\nIf your project is using git be sure to run the command in the correct directory.\nIf your project does not use git, run again with the --offline option.");
        exit(1);
    }

    Ok(())
}
