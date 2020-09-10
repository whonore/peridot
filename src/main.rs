use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

mod cli;
mod link;
mod output;
mod path;

use cli::{AppConfig, Cli, Config};
use link::{check_link, make_link, LinkStatus};
use output::AppOutput;
use LinkStatus::*;

fn link(base_dir: &PathBuf, name: &str, app: &AppConfig, check_only: bool) -> Result<AppOutput> {
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
    Ok(out)
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let config = Config::new(args)?;

    // TODO: multithreading
    for (name, app) in config.apps {
        let out = link(&config.base_dir, &name, &app, config.check_only)?;
        println!("{}", out);
    }
    Ok(())
}
