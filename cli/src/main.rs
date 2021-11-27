#[macro_use]
extern crate lazy_static;
use clap::Parser;
use directories::ProjectDirs;
mod pie;

#[derive(Parser, Debug)]
#[clap(version = "0.1.0-alpha.1", author = "beni69 <beni@karesz.xyz>")]
struct Cli {
    #[clap(short, long, default_value = "http://127.0.0.1:6969")]
    url: String,
    #[clap(subcommand)]
    subcmd: SubCommand,
}
#[derive(Parser, Debug)]
enum SubCommand {
    Greet,
    Ping,
    Deploy(Deploy),
}
#[derive(Parser, Clone, Copy, Debug)]
pub struct Deploy {
    #[clap(short, long)]
    offline: bool,
}

lazy_static! {
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "beni69", "pie").unwrap();
    static ref CLI: Cli = Cli::parse();
}

#[derive(Debug)]
enum MainError {
    SurfError(surf::Error),
}
impl From<surf::Error> for MainError {
    fn from(e: surf::Error) -> Self {
        Self::SurfError(e)
    }
}

#[async_std::main]
async fn main() -> Result<(), MainError> {
    match CLI.subcmd {
        SubCommand::Greet => println!("Hello, world!"),
        SubCommand::Ping => pie::ping().await?,
        SubCommand::Deploy(opts) => pie::deploy(opts).await?,
    }

    Ok(())
}
