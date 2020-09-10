use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

type Apps = HashMap<String, AppConfig>;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct AppsWrap(Apps);

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub dir: Option<String>,
    pub description: Option<String>,
    pub links: Option<Vec<(String, String)>>,
}

// TODO: allow specifying specific apps
#[derive(Debug, StructOpt)]
#[structopt(name = "dotty", about = "A dotfile manager")]
pub struct Cli {
    #[structopt(parse(from_os_str))]
    base_dir: Option<PathBuf>,
    #[structopt(short = "c", long = "config-file", parse(from_os_str))]
    config_file: Option<PathBuf>,
    #[structopt(short = "C", long = "check-only")]
    check_only: bool,
}

fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap())
}

fn find_config(base_dir: &Path) -> PathBuf {
    base_dir.join("dotty.toml")
}

#[derive(Debug)]
pub struct Config {
    pub base_dir: PathBuf,
    pub apps: Apps,
    pub check_only: bool,
}

impl Config {
    pub fn new(args: Cli) -> Result<Config> {
        let base_dir = args
            .base_dir
            .unwrap_or_else(|| home_dir().join(".dotfiles"))
            .canonicalize()?;
        let config_file = args
            .config_file
            .unwrap_or_else(|| find_config(&base_dir))
            .canonicalize()?;
        let apps: AppsWrap = toml::from_str(&std::fs::read_to_string(&config_file)?)?;

        Ok(Config {
            base_dir,
            apps: apps.0,
            check_only: args.check_only,
        })
    }
}
