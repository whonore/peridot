use anyhow::Result;
use std::path::Path;
use structopt::StructOpt;

mod cli;
mod link;
mod output;
mod path;

use cli::{AppConfig, Cli, Config};
use link::{check_link, make_link, LinkStatus};
use output::AppOutput;
use path::eval_env;
use LinkStatus::*;

fn link(base_dir: &Path, name: &str, app: &AppConfig, check_only: bool) -> Result<AppOutput> {
    let mut out = AppOutput::new(name);
    let dstdir = eval_env(&base_dir.join(app.dstdir.as_deref().unwrap_or(name)))?;
    let srcdir = eval_env(Path::new(app.srcdir.as_deref().unwrap_or("$HOME")))?;

    if let Some(links) = &app.links {
        for link in links {
            match check_link(&dstdir, &srcdir, link) {
                Ok(link) => match link.status {
                    SrcUnexists => {
                        if !check_only {
                            match make_link(link.src.clone(), link.dst.clone()) {
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
