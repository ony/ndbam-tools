use std::io;
use std::path::PathBuf;

pub use crate::utils::atomic_file::AtomicSession;
use crate::utils::atomic_file::*;

pub struct ContentsWriter<T: io::Write = AtomicFile>(T);

pub fn create(path: PathBuf) -> io::Result<ContentsWriter<AtomicFile>> {
    Ok(ContentsWriter(AtomicFile::create(path)?))
}

impl<T: io::Write + AtomicSession> AtomicSession for ContentsWriter<T> {
    type AtomicResult = T::AtomicResult;

    #[inline]
    fn commit(self) -> Self::AtomicResult {
        self.0.commit()
    }
}
