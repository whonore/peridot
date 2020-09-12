use std::fs;
use std::os::unix;
use std::path::{Path, PathBuf};
use std::result;

use crate::cli::Apps;
use crate::path::{resolve_env, resolve_name, PathError};

#[derive(Debug)]
pub enum LinkStatus {
    SrcUnexists,
    DstUnexists,
    Exists,
    Unexpected(PathBuf),
}

use LinkStatus::*;

#[derive(Debug)]
pub struct Link {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub status: LinkStatus,
}

impl Link {
    fn src_unexists(src: PathBuf, dst: PathBuf) -> Self {
        Link {
            src,
            dst,
            status: SrcUnexists,
        }
    }

    fn dst_unexists(src: PathBuf, dst: PathBuf) -> Self {
        Link {
            src,
            dst,
            status: DstUnexists,
        }
    }

    fn exists(src: PathBuf, dst: PathBuf) -> Self {
        Link {
            src,
            dst,
            status: Exists,
        }
    }

    fn unexpected(src: PathBuf, dst: PathBuf, expected: PathBuf) -> Self {
        Link {
            src,
            dst,
            status: Unexpected(expected),
        }
    }
}

pub fn check_link(
    apps: &Apps,
    dstdir: &Path,
    srcdir: &Path,
    link: &(String, String),
) -> result::Result<Link, PathError> {
    let (dst, src) = link;
    let dst = dstdir.join(resolve_name(
        &|name| apps.resolve_name(name),
        &resolve_env(Path::new(dst))?,
    )?);
    let src = srcdir.join(resolve_env(Path::new(src))?);

    if src.exists() {
        let real_dst = src.read_link()?;
        if dst == real_dst {
            Ok(Link::exists(src, dst))
        } else {
            Ok(Link::unexpected(src, dst, real_dst))
        }
    } else if !dst.exists() {
        Ok(Link::dst_unexists(src, dst))
    } else {
        Ok(Link::src_unexists(src, dst))
    }
}

pub fn make_link(src: PathBuf, dst: PathBuf) -> result::Result<Link, PathError> {
    let dir = src
        .parent()
        .ok_or_else(|| PathError::NoParent(src.display().to_string()))?;
    fs::create_dir_all(dir)?;
    unix::fs::symlink(&dst, &src)?;
    Ok(Link::exists(src, dst))
}
