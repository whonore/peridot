use anyhow::Result;
use std::fmt;
use std::path::PathBuf;
use structopt::StructOpt;

mod cli;
mod link;
mod path;

use cli::{AppConfig, Cli, Config};
use link::{check_link, make_link, Link, LinkStatus};
use path::PathError;
use LinkStatus::*;

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
                    _ => out.add_link(&link),
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
