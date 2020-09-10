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

static SUCCESS: &str = "✓";
static FAILURE: &str = "❌";
static LINKSTO: &str = "→";
static NOTLINKSTO: &str = "↛";
static EDGE: &str = "├";
static STRAIGHT: &str = "│";
static CORNER: &str = "└";

#[derive(Debug)]
enum AppResult {
    Ok(Link),
    Err {
        error: PathError,
        link: Option<(PathBuf, PathBuf)>,
    },
}

impl AppResult {
    fn display_link(src: &PathBuf, dst: &PathBuf) -> Vec<String> {
        vec![format!(
            "{}─ {} {} {}",
            SUCCESS,
            src.display(),
            LINKSTO,
            dst.display()
        )]
    }

    fn display_notlink(src: &PathBuf, dst: &PathBuf, err: Option<&str>) -> Vec<String> {
        let mut lines = vec![format!(
            "{}─ {} {} {}",
            FAILURE,
            src.display(),
            NOTLINKSTO,
            dst.display()
        )];
        if let Some(err) = err {
            lines.push(format!("   Error: {}", err));
        };
        lines
    }

    fn lines(&self) -> Vec<String> {
        match self {
            AppResult::Ok(Link { src, dst, status }) => match status {
                SrcUnexists => AppResult::display_notlink(src, dst, None),
                DstUnexists => AppResult::display_notlink(src, dst, Some("target does not exist")),
                Exists => AppResult::display_link(src, dst),
                Unexpected(found) => AppResult::display_notlink(
                    src,
                    dst,
                    Some(&format!("found {}", found.display())),
                ),
            },
            AppResult::Err {
                error,
                link: Some((src, dst)),
            } => AppResult::display_notlink(src, dst, Some(&format!("{}", error))),
            AppResult::Err { error, link: None } => vec![format!("{}─ Error: {}", FAILURE, error)],
        }
    }

    fn display(&self, f: &mut fmt::Formatter, edge: &str) -> fmt::Result {
        if let Some((first, rest)) = self.lines().split_first() {
            write!(f, "\n{}─{}", edge, first)?;
            rest.iter()
                .map(|res| write!(f, "\n{}  {}", STRAIGHT, res))
                .collect::<fmt::Result>()?;
        };
        Ok(())
    }
}

#[derive(Debug)]
struct AppOutput {
    name: String,
    results: Vec<AppResult>,
}

impl AppOutput {
    fn new(name: &str) -> Self {
        AppOutput {
            name: name.into(),
            results: Vec::new(),
        }
    }

    fn output_link(&mut self, res: Link) {
        self.results.push(AppResult::Ok(res))
    }

    fn output_error(&mut self, error: PathError, link: Option<(PathBuf, PathBuf)>) {
        self.results.push(AppResult::Err { error, link })
    }
}

impl fmt::Display for AppOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: put name in a box
        write!(f, "{}", self.name)?;
        if let Some((last, results)) = self.results.split_last() {
            for res in results {
                res.display(f, EDGE)?
            }
            last.display(f, CORNER)?;
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
                                Ok(link) => out.output_link(link),
                                Err(e) => out.output_error(e, Some((link.src, link.dst))),
                            }
                        } else {
                            out.output_link(link)
                        }
                    }
                    _ => out.output_link(link),
                },
                Err(e) => out.output_error(e, None),
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
