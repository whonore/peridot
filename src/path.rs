use std::env;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum PathError {
    InvalidEnvVar { path: PathBuf, env: String },
    NoParent(String),
    IoError(io::Error),
}

use PathError::*;

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidEnvVar { path, env } => write!(
                f,
                "Could not find environment variable {} in {}",
                env,
                path.display()
            ),
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

// TODO: allow referencing other app dirs
pub fn eval_env(path: &Path) -> std::result::Result<PathBuf, PathError> {
    path.iter()
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
