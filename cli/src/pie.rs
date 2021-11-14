use crate::CLI;
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
