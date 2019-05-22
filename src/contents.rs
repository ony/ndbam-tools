mod parser;
mod writer;

use std::collections::HashMap;
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

    pub fn mtime(&self) -> Option<&SystemTime> {
        match self {
            Entry::Dir { .. } => None,
            Entry::File { mtime, .. } => Some(mtime),
            Entry::Sym { mtime, .. } => Some(mtime),
        }
    }
}
