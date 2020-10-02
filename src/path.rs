use std::env;
use std::error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::result;

#[derive(Debug)]
pub enum PathError {
    InvalidEnvVar(shellexpand::LookupError<env::VarError>),
    InvalidNameRef { path: PathBuf, name: String },
    NoParent(String),
    IoError(io::Error),
}

use PathError::*;

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidEnvVar(e) => write!(f, "{}", e),
            InvalidNameRef { path, name } => {
                write!(f, "Invalid name reference {} in {}", name, path.display())
            }
            NoParent(path) => write!(f, "{} must have a parent directory", path),
            IoError(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for PathError {}

impl From<io::Error> for PathError {
    fn from(e: io::Error) -> Self {
        IoError(e)
    }
}

impl From<shellexpand::LookupError<env::VarError>> for PathError {
    fn from(e: shellexpand::LookupError<env::VarError>) -> Self {
        InvalidEnvVar(e)
    }
}

pub fn resolve_env(path: &Path) -> result::Result<PathBuf, PathError> {
    shellexpand::full(&path.display().to_string())
        .map(|p| PathBuf::from(p.to_string()))
        .map_err(|e| e.into())
}

pub fn resolve_name<F>(lookup: &F, path: &Path) -> Result<PathBuf, PathError>
where
    F: Fn(&str) -> Option<PathBuf>,
{
    path.iter()
        .map(|comp| {
            let comp = comp.to_string_lossy();
            if comp.starts_with("{{") && comp.ends_with("}}") {
                lookup(&comp[2..comp.len() - 2]).ok_or_else(|| InvalidNameRef {
                    path: path.into(),
                    name: comp.into(),
                })
            } else {
                Ok(PathBuf::from(comp.to_string()))
            }
        })
        .collect()
}
