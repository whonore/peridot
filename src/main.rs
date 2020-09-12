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

fn link(apps: &Apps, name: &str, app: &App, check_only: bool) -> Result<AppOutput> {
    let mut out = AppOutput::new(name);

    for link in &app.links {
        match check_link(apps, &app.dstdir, &app.srcdir, link) {
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
    Ok(out)
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let config = Config::new(args)?;

    // TODO: multithreading
    for (name, app) in &config.apps.0 {
        let out = link(&config.apps, &name, &app, config.check_only)?;
        println!("{}", out);
    }
    Ok(())
}
