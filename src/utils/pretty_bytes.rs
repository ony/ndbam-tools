use crate::utils::semi_binary::*;
use std::fmt;

#[derive(PartialEq)]
pub struct PrettySlice<'s>(&'s [u8]);

impl<'s> fmt::Debug for PrettySlice<'s> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\"")?;
        for chunk in iter_utf8_chunks(self.0) {
            match chunk {
                Utf8Chunk::Valid(chars) => write!(f, "{}", chars.escape_debug())?,
                Utf8Chunk::Bytes(bytes) => {
                    for byte in bytes {
                        write!(f, "\\x{:02x}", byte)?;
                    }
                }
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}

#[derive(PartialEq)]
struct PrettyBytes<T: AsRef<[u8]>>(T);

impl<T: AsRef<[u8]>> fmt::Debug for PrettyBytes<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.as_ref().pretty().fmt(f)
    }
}
pub trait AsPrettySlice {
    fn pretty(&self) -> PrettySlice;
}

impl AsPrettySlice for Vec<u8> {
    fn pretty(&self) -> PrettySlice {
        PrettySlice(self)
    }
}

impl AsPrettySlice for [u8] {
    fn pretty(&self) -> PrettySlice {
        PrettySlice(self)
    }
}
