pub use crypto_hash::Algorithm;
use crypto_hash::Hasher;
use std::io::prelude::*;
use std::path::Path;
use std::{fs, io};

pub fn file_hash<P: AsRef<Path>>(algorithm: Algorithm, path: P) -> io::Result<String> {
    let mut reader = io::BufReader::new(fs::File::open(path.as_ref())?);
    let mut hasher = Hasher::new(algorithm);
    loop {
        let chunk = reader.fill_buf()?;
        if chunk.is_empty() {
            return Ok(hex::encode(hasher.finish()));
        }
        hasher.write_all(chunk)?;

        let n = chunk.len();
        reader.consume(n);
    }
}
