#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
use directories::ProjectDirs;

pub mod config;
pub mod utils;

lazy_static! {
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "beni69", "pie").unwrap();
}
