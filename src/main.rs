use anyhow::Result;
use structopt::StructOpt;

mod cli;
mod link;
mod output;
mod path;

use cli::{App, Apps, Cli, Config};
use link::{check_link, make_link, LinkStatus};
use output::AppOutput;
use LinkStatus::*;

fn link(apps: &Apps, name: &str, app: &App, do_link: bool) -> Result<AppOutput> {
    let mut out = AppOutput::new(name);

    for link in &app.links {
        match check_link(apps, &app.dstdir, &app.srcdir, link) {
            Ok(link) => match link.status {
                SrcUnexists => {
                    if do_link {
                        match make_link(link.src.clone(), link.dst.clone()) {
                            Ok(link) => out.link(link),
                            Err(e) => out.error(e, Some((link.src, link.dst))),
                        }
                    } else {
                        out.link(link)
                    }
                }
                _ => out.link(link),
            },
            Err(e) => out.error(e, None),
        }
    }
    Ok(out)
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let config = Config::new(args)?;

    // TODO: multithreading
    for (name, app) in &config.apps.0 {
        let out = link(&config.apps, &name, &app, config.link)?;
        println!("{}", out);
    }
    Ok(())
}
