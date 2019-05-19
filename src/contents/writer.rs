use std::io;
use std::path::PathBuf;

use crate::utils::atomic_file::*;

pub struct ContentsWriter(AtomicFile);

impl ContentsWriter {
    pub fn create(path: PathBuf) -> io::Result<ContentsWriter> {
        Ok(ContentsWriter(AtomicFile::create(path)?))
    }

    #[inline]
    pub fn commit(self) -> io::Result<()> {
        self.0.commit()
    }
}
