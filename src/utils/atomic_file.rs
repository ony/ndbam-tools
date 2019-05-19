use std::path::PathBuf;
use std::{fs, io};

use tempfile::NamedTempFile;

pub struct AtomicFile {
    path: PathBuf,
    temp: NamedTempFile,
}

impl AtomicFile {
    /// Opens a file in write-only mode.
    ///
    /// Later call to 'commit' will either create file if it doesn't exist or replace its content.
    #[inline]
    pub fn create(path: PathBuf) -> io::Result<Self> {
        let temp = NamedTempFile::new_in(path.parent().expect("invalid path to file"))?;
        Ok(AtomicFile { path, temp })
    }

    /// Actually replace/create file with content that were written already
    pub fn commit(self) -> io::Result<()> {
        self.temp
            .into_temp_path()
            .persist(self.path)
            .map_err(|e| io::Error::new(e.error.kind(), e))
    }
}

impl AsRef<fs::File> for AtomicFile {
    #[inline]
    fn as_ref(&self) -> &fs::File {
        self.temp.as_file()
    }
}

impl AsMut<fs::File> for AtomicFile {
    #[inline]
    fn as_mut(&mut self) -> &mut fs::File {
        self.temp.as_file_mut()
    }
}

impl io::Write for AtomicFile {
    #[inline]
    fn write(&mut self, chunk: &[u8]) -> io::Result<usize> {
        self.temp.write(chunk)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.temp.flush()
    }
}
