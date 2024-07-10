use std::{
    io,
    path::{Path, PathBuf},
};

use args::Args;
use clap::Parser;
use dirs::home_dir;

mod args;
mod env;
mod error;
mod site;
mod types;
use error::AppResult;
use site::Site;

fn main() -> AppResult<()> {
    let args = Args::parse();
    Site::new(Path::new(&args.dir).to_path_buf(), get_home_dir()?)
        .detect()?
        .get_credentials()?
        .db_backup()?
        .files_backup()?;

    Ok(())
}

fn get_home_dir() -> AppResult<PathBuf> {
    Ok(home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?)
}
