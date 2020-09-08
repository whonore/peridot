use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::os::unix;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

type Apps = HashMap<String, AppConfig>;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct AppsWrap(Apps);

type LinkPair = (String, String);

#[derive(Debug, Deserialize)]
struct AppConfig {
    dir: Option<String>,
    description: Option<String>,
    links: Option<Vec<LinkPair>>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "dotty", about = "A dotfile manager")]
struct Cli {
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
struct Config {
    base_dir: PathBuf,
    apps: Apps,
    check_only: bool,
}

impl Config {
    fn new(args: Cli) -> Result<Config> {
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

#[derive(Debug)]
struct AppOutput {
    name: String,
    // TODO: use Ok, Fail, ? type and push formatting to Display
    results: Vec<String>,
}

impl AppOutput {
    fn new(name: &str) -> Self {
        AppOutput {
            name: name.into(),
            results: Vec::new(),
        }
    }

    fn add_link(&mut self, link: &Link) {
        self.results.push(match &link.status {
            SrcUnexists => format!("?─ {} ↛ {}", link.src.display(), link.dst.display()),
            DstUnexists => format!(
                "❌─ {} ↛ {} (Failed: target does not exist)",
                link.src.display(),
                link.dst.display()
            ),
            Exists => format!("✓─ {} → {}", link.src.display(), link.dst.display()),
            Unexpected(found) => format!(
                "❌─ {} ↛ {} (expected: {})",
                link.src.display(),
                found.display(),
                link.dst.display()
            ),
        })
    }

    fn add_error(&mut self, error: &PathError, src: Option<&PathBuf>, dst: Option<&PathBuf>) {
        if src.is_none() || dst.is_none() {
            self.results.push(format!("❌─ (Failed: {})", error))
        } else {
            self.results.push(format!(
                "❌─ {} ↛ {} (Failed: {})",
                src.unwrap().display(),
                dst.unwrap().display(),
                error
            ))
        }
    }
}

impl fmt::Display for AppOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: put name in a box
        write!(f, "{}", self.name)?;
        if let Some((last, results)) = self.results.split_last() {
            results
                .iter()
                .map(|res| write!(f, "\n├──{}", res))
                .collect::<fmt::Result>()?;
            write!(f, "\n└──{}", last)?;
        }
        writeln!(f)
    }
}

#[derive(Debug)]
enum PathError {
    InvalidEnvVar { path: String, env: String },
    NoParent(String),
    IoError(io::Error),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidEnvVar { path, env } => {
                write!(f, "Could not find environment variable {} in {}", env, path)
            }
            NoParent(path) => write!(f, "{} must have a parent directory", path),
            IoError(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for PathError {
    fn from(e: io::Error) -> Self {
        IoError(e)
    }
}

#[derive(Debug)]
enum LinkStatus {
    SrcUnexists,
    DstUnexists,
    Exists,
    Unexpected(PathBuf),
}

#[derive(Debug)]
struct Link {
    src: PathBuf,
    dst: PathBuf,
    status: LinkStatus,
}

impl Link {
    fn src_unexists(src: PathBuf, dst: PathBuf) -> Self {
        Link {
            src,
            dst,
            status: SrcUnexists,
        }
    }

    fn dst_unexists(src: PathBuf, dst: PathBuf) -> Self {
        Link {
            src,
            dst,
            status: DstUnexists,
        }
    }

    fn exists(src: PathBuf, dst: PathBuf) -> Self {
        Link {
            src,
            dst,
            status: Exists,
        }
    }

    fn unexpected(src: PathBuf, dst: PathBuf, expected: PathBuf) -> Self {
        Link {
            src,
            dst,
            status: Unexpected(expected),
        }
    }
}

use LinkStatus::*;
use PathError::*;

fn to_path(path: &str) -> std::result::Result<PathBuf, PathError> {
    Path::new(path)
        .iter()
        .map(|comp| {
            let comp = comp.to_str().unwrap();
            if comp.starts_with('$') {
                env::var(&comp[1..]).or_else(|_| {
                    Err(InvalidEnvVar {
                        path: path.into(),
                        env: comp.into(),
                    })
                })
            } else {
                Ok(comp.into())
            }
        })
        .collect()
}

fn check_link(dir: &PathBuf, link: &LinkPair) -> std::result::Result<Link, PathError> {
    let (dst, src) = link;
    let dst = dir.join(to_path(dst)?);
    let src = to_path(src)?;

    if src.exists() {
        let real_dst = src.read_link()?;
        if dst == real_dst {
            Ok(Link::exists(src, dst))
        } else {
            Ok(Link::unexpected(src, dst, real_dst))
        }
    } else if !dst.exists() {
        Ok(Link::dst_unexists(src, dst))
    } else {
        Ok(Link::src_unexists(src, dst))
    }
}

fn make_link(src: &PathBuf, dst: &PathBuf) -> std::result::Result<Link, PathError> {
    let dir = src
        .parent()
        .ok_or_else(|| NoParent(src.display().to_string()))?;
    fs::create_dir_all(dir)?;
    unix::fs::symlink(dst, src)?;
    Ok(Link::exists(src.clone(), dst.clone()))
}

fn link(base_dir: &PathBuf, name: &str, app: &AppConfig, check_only: bool) -> Result<()> {
    let mut out = AppOutput::new(name);
    let dir = base_dir
        .join(app.dir.as_deref().unwrap_or(name))
        .canonicalize()?;
    if let Some(links) = &app.links {
        for link in links {
            match check_link(&dir, link) {
                Ok(link) => match link.status {
                    SrcUnexists => {
                        if !check_only {
                            match make_link(&link.src, &link.dst) {
                                Ok(link) => out.add_link(&link),
                                Err(e) => out.add_error(&e, Some(&link.src), Some(&link.dst)),
                            }
                        } else {
                            out.add_link(&link)
                        }
                    }
                    DstUnexists => out.add_link(&link),
                    Exists => out.add_link(&link),
                    Unexpected(_) => out.add_link(&link),
                },
                Err(e) => out.add_error(&e, None, None),
            }
        }
    };
    println!("{}", out);
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let config = Config::new(args)?;

    config
        .apps
        .iter()
        .map(|(name, app)| link(&config.base_dir, name, app, config.check_only))
        .collect()
}
