#[macro_use]
extern crate lazy_static;
use directories::ProjectDirs;

pub mod config;

lazy_static! {
    static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::from("", "beni69", "pie").unwrap();
}
