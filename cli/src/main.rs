#[macro_use]
extern crate lazy_static;
use clap::Parser;
use directories::ProjectDirs;
mod pie;

#[derive(Parser)]
#[clap(version = "0.1.0-alpha.1", author = "beni69 <beni@karesz.xyz>")]
struct Cli {
    #[clap(short, long, default_value = "http://127.0.0.1:6969")]
    url: String,
    #[clap(subcommand)]
    subcmd: SubCommand,
}
#[derive(Parser)]
enum SubCommand {
    Greet,
    Ping,
}

lazy_static! {
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "beni69", "pie").unwrap();
    static ref CLI: Cli = Cli::parse();
}

#[async_std::main]
async fn main() {
    match CLI.subcmd {
        SubCommand::Greet => println!("Hello, world!"),
        SubCommand::Ping => pie::ping().await.unwrap(),
    }
}
