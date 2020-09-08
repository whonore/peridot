use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct Config(HashMap<String, AppConfig>);

type Link = (String, String);

#[derive(Debug, Deserialize)]
struct AppConfig {
    dir: Option<String>,
    description: Option<String>,
    links: Option<Vec<Link>>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "dotty", about = "A dotfile manager")]
struct Cli {
    #[structopt(parse(from_os_str))]
    dotfile_dir: Option<PathBuf>,
    #[structopt(short = "c", long = "config-file", parse(from_os_str))]
    config_file: Option<PathBuf>,
}

fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap())
}

fn find_config(dotfiles: &Path) -> PathBuf {
    dotfiles.join("dotty.toml")
}

fn parse_config(args: Cli) -> Result<(PathBuf, Config)> {
    let dotfile_dir = args
        .dotfile_dir
        .unwrap_or_else(|| home_dir().join(".dotfiles"))
        .canonicalize()?;
    let config_file = args
        .config_file
        .unwrap_or_else(|| find_config(&dotfile_dir))
        .canonicalize()?;
    let config: Config = toml::from_str(&std::fs::read_to_string(&config_file)?)?;

    Ok((dotfile_dir, config))
}

fn to_path(path: &str) -> Result<PathBuf> {
    Path::new(path)
        .iter()
        .map(|comp| {
            let comp = comp
                .to_str()
                .ok_or_else(|| anyhow!("Can't convert {} to a str", comp.to_string_lossy()))?;
            if comp.starts_with('$') {
                env::var(&comp[1..]).or_else(|_| Err(anyhow!("Can't find {}", comp)))
            } else {
                Ok(comp.into())
            }
        })
        .collect()
}

#[derive(Debug)]
struct AppOutput {
    name: String,
    results: Vec<String>,
}

impl AppOutput {
    fn new(name: &str) -> Self {
        AppOutput {
            name: name.into(),
            results: Vec::new(),
        }
    }

    fn add_result(&mut self, res: String) {
        self.results.push(res)
    }
}

impl fmt::Display for AppOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some((last, results)) = self.results.split_last() {
            results
                .iter()
                .map(|res| write!(f, "\n├── {}", res))
                .collect::<fmt::Result>()?;
            write!(f, "\n└── {}", last)?;
        }
        writeln!(f)
    }
}

fn check_link(out: &mut AppOutput, dir: &PathBuf, link: &Link) -> Result<()> {
    let (to, from) = link;
    let to = dir.join(to_path(to)?);
    let from = to_path(from)?;
    let real_to = fs::read_link(&from)?;

    if to == real_to {
        out.add_result(format!("✓  {} → {}", from.display(), to.display()))
    } else {
        out.add_result(format!(
            "❌  {} → {} (expected {})",
            from.display(),
            real_to.display(),
            to.display()
        ))
    };
    Ok(())
}

fn check(dotfile: &PathBuf, name: &str, app: &AppConfig) -> Result<()> {
    let mut out = AppOutput::new(name);
    let dir = dotfile
        .join(app.dir.as_deref().unwrap_or(name))
        .canonicalize()?;
    if let Some(links) = &app.links {
        for link in links {
            check_link(&mut out, &dir, link)?
        }
    };
    println!("{}", out);
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let (dotfile_dir, config) = parse_config(args)?;

    config
        .0
        .iter()
        .map(|(name, app)| check(&dotfile_dir, name, app))
        .collect()
}
