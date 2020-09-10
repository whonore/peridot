use std::fmt;
use std::path::PathBuf;

use crate::link::{Link, LinkStatus};
use crate::path::PathError;
use LinkStatus::*;

static SUCCESS: &str = "✓";
static FAILURE: &str = "❌";
static LINKSTO: &str = "→";
static NOTLINKSTO: &str = "↛";

static TREE_EDGE: &str = "├";
static TREE_VERT: &str = "│";
static TREE_CORNER: &str = "└";

static TITLE_TLCORNER: &str = "╔";
static TITLE_TRCORNER: &str = "╗";
static TITLE_BLCORNER: &str = "╚";
static TITLE_BRCORNER: &str = "╝";
static TITLE_VERT: &str = "║";
static TITLE_HORZ: &str = "═";

struct Title<'a>(&'a str);

impl fmt::Display for Title<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w = self.0.len();
        writeln!(
            f,
            "{}{}{}",
            TITLE_TLCORNER,
            TITLE_HORZ.repeat(w + 2),
            TITLE_TRCORNER
        )?;
        writeln!(f, "{} {} {}", TITLE_VERT, self.0, TITLE_VERT)?;
        write!(
            f,
            "{}{}{}",
            TITLE_BLCORNER,
            TITLE_HORZ.repeat(w + 2),
            TITLE_BRCORNER
        )
    }
}

#[derive(Debug)]
enum AppResult {
    Ok(Link),
    Err {
        error: PathError,
        link: Option<(PathBuf, PathBuf)>,
    },
}

// TODO: colorize output
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
                .map(|res| write!(f, "\n{}  {}", TREE_VERT, res))
                .collect::<fmt::Result>()?;
        };
        Ok(())
    }
}

#[derive(Debug)]
pub struct AppOutput {
    name: String,
    results: Vec<AppResult>,
}

impl AppOutput {
    pub fn new(name: &str) -> Self {
        AppOutput {
            name: name.into(),
            results: Vec::new(),
        }
    }

    pub fn output_link(&mut self, res: Link) {
        self.results.push(AppResult::Ok(res))
    }

    pub fn output_error(&mut self, error: PathError, link: Option<(PathBuf, PathBuf)>) {
        self.results.push(AppResult::Err { error, link })
    }
}

impl fmt::Display for AppOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Title(&self.name))?;
        if let Some((last, results)) = self.results.split_last() {
            for res in results {
                res.display(f, TREE_EDGE)?
            }
            last.display(f, TREE_CORNER)?;
        }
        writeln!(f)
    }
}
