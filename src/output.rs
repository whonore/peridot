use ansi_term::Color;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::link::{Link, LinkStatus};
use crate::path::PathError;
use LinkStatus::*;

static SUCCESS: &str = "✓";
static FAILURE: &str = "❌";
static LINKSTO: &str = "→";
static NOTLINKSTO: &str = "↛";

static TREE_EDGE: &str = "├";
static TREE_VERT: &str = "│";
static TREE_HORZ: &str = "─";
static TREE_CORNER: &str = "└";

static TITLE_TLCORNER: &str = "╔";
static TITLE_TRCORNER: &str = "╗";
static TITLE_BLCORNER: &str = "╚";
static TITLE_BRCORNER: &str = "╝";
static TITLE_VERT: &str = "║";
static TITLE_HORZ: &str = "═";

#[derive(Debug)]
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
        writeln!(
            f,
            "{} {} {}",
            TITLE_VERT,
            Color::Blue.bold().paint(self.0),
            TITLE_VERT
        )?;
        write!(
            f,
            "{}{}{}",
            TITLE_BLCORNER,
            TITLE_HORZ.repeat(w + 2),
            TITLE_BRCORNER
        )
    }
}

struct AppError<'a>(&'a dyn fmt::Display);

impl fmt::Display for AppError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}",
            Color::Red.underline().bold().paint("Error:"),
            self.0
        )
    }
}

#[derive(Debug)]
struct AppLink<'a>(&'a Path);

impl fmt::Display for AppLink<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let color = if self.0.exists() {
            Color::Cyan
        } else {
            Color::Red
        };
        write!(f, "{}", color.paint(self.0.display().to_string()))
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

impl AppResult {
    fn display_link(src: &Path, dst: &Path) -> Vec<String> {
        vec![format!(
            "{}{} {} {} {}",
            Color::Green.paint(SUCCESS),
            TREE_HORZ,
            AppLink(src),
            Color::Green.paint(LINKSTO),
            AppLink(dst)
        )]
    }

    fn display_notlink(src: &Path, dst: &Path, err: &dyn fmt::Display) -> Vec<String> {
        vec![
            format!(
                "{}{} {} {} {}",
                Color::Red.paint(FAILURE),
                TREE_HORZ,
                AppLink(src),
                Color::Red.paint(NOTLINKSTO),
                AppLink(dst),
            ),
            format!("  {}", AppError(err)),
        ]
    }

    fn lines(&self) -> Vec<String> {
        match self {
            AppResult::Ok(Link { src, dst, status }) => match status {
                SrcUnexists => {
                    AppResult::display_notlink(src, dst, &"link does not exist".to_string())
                }
                DstUnexists => {
                    AppResult::display_notlink(src, dst, &"target does not exist".to_string())
                }
                Exists => AppResult::display_link(src, dst),
                Unexpected(found) => {
                    AppResult::display_notlink(src, dst, &format!("found {}", AppLink(found)))
                }
            },
            AppResult::Err {
                error,
                link: Some((src, dst)),
            } => AppResult::display_notlink(src, dst, error),
            AppResult::Err { error, link: None } => vec![format!(
                "{}{} {}",
                Color::Red.paint(FAILURE),
                TREE_HORZ,
                AppError(error)
            )],
        }
    }

    fn display(&self, f: &mut fmt::Formatter, last: bool) -> fmt::Result {
        if let Some((first, rest)) = self.lines().split_first() {
            let edge = if last { TREE_CORNER } else { TREE_EDGE };
            write!(f, "\n{}{}{}", edge, TREE_HORZ, first)?;
            for res in rest {
                let edge = if last { " " } else { TREE_VERT };
                write!(f, "\n{}  {}", edge, res)?;
            }
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
                res.display(f, false)?
            }
            last.display(f, true)?;
        }
        writeln!(f)
    }
}
