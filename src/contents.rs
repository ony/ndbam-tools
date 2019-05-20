mod parser;
mod writer;

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub use crate::utils::hashing::*;
pub use parser::*;
pub use writer::*;

/// Represents NDBAM/VDB contents entry
///
#[derive(Debug, PartialEq)]
pub enum Entry {
    Dir { path: PathBuf },
    File { path: PathBuf, md5: String, mtime: SystemTime, extra: HashMap<String, String> },
    Sym { path: PathBuf, target: PathBuf, mtime: SystemTime, extra: HashMap<String, String> },
}

impl Entry {
    pub fn path(&self) -> &Path {
        match self {
            Entry::Dir { path, .. } => path,
            Entry::File { path, .. } => path,
            Entry::Sym { path, .. } => path,
        }
    }

    pub fn path_in(&self, root: &Path) -> PathBuf {
        let mut components = self.path().components();
        components.next();
        root.join(components.as_path())
    }

    pub fn mtime(&self) -> Option<&SystemTime> {
        match self {
            Entry::Dir { .. } => None,
            Entry::File { mtime, .. } => Some(mtime),
            Entry::Sym { mtime, .. } => Some(mtime),
        }
    }

    pub fn from_path(real_path: &Path, root: &Path) -> io::Result<Entry> {
        let mut path = PathBuf::from("/");
        path.push(real_path.strip_prefix(root).unwrap());

        let metadata = real_path.symlink_metadata()?;
        if metadata.is_dir() {
            Ok(Entry::Dir { path })
        } else if metadata.is_file() {
            Ok(Entry::File {
                path,
                md5: file_hash(Algorithm::MD5, real_path)?,
                mtime: metadata.modified()?,
                extra: Default::default(),
            })
        } else {
            Ok(Entry::Sym {
                path,
                target: real_path.read_link()?,
                mtime: metadata.modified()?,
                extra: Default::default(),
            })
        }
    }
}
