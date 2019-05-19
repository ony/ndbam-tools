use std::collections::HashMap;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub use crate::utils::atomic_file::AtomicSession;
use crate::utils::atomic_file::*;
use crate::utils::semi_binary::*;

use super::Entry;

pub struct ContentsWriter<T: io::Write = AtomicFile>(T);

pub fn create(path: PathBuf) -> io::Result<ContentsWriter<AtomicFile>> {
    Ok(ContentsWriter(AtomicFile::create(path)?))
}

impl<T: io::Write> ContentsWriter<T> {
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn from_root(&mut self, root: &Path) -> io::Result<()> {
        for node in WalkDir::new(root) {
            let node = node.unwrap();
            self.write_entry(&Entry::from_path(node.path(), root)?)?;
        }
        Ok(())
    }

    pub fn write_entry(&mut self, entry: &Entry) -> io::Result<()> {
        match entry {
            Entry::Dir { path } => {
                println!("dir path: {:?}", path);
                self.write_raw("type=dir path=")?;
                self.write_escaped_os_str(dbg!(path.as_os_str()))?;
            }
            Entry::File {
                path,
                md5,
                mtime,
                extra,
            } => {
                println!("file path: {:?}", path);
                self.write_raw("type=file path=")?;
                self.write_escaped_os_str(path.as_os_str())?;
                self.write_raw(" md5=")?;
                self.write_raw(md5)?;
                self.write_raw(" mtime=")?;
                self.write_mtime(&mtime)?;
                self.write_extra_tokens(&extra)?;
            }
            Entry::Sym {
                path,
                target,
                mtime,
                extra,
            } => {
                println!("sym path: {:?}", path);
                self.write_raw("type=sym path=")?;
                self.write_escaped_os_str(path.as_os_str())?;
                self.write_raw(" target=")?;
                self.write_escaped_os_str(target.as_os_str())?;
                self.write_raw(" mtime=")?;
                self.write_mtime(&mtime)?;
                self.write_extra_tokens(&extra)?;
            }
        }
        self.0.write_all(b"\n")?;
        Ok(())
    }

    fn write_mtime(&mut self, mtime: &SystemTime) -> io::Result<()> {
        self.write_raw(
            &mtime
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
        )
    }

    fn write_extra_tokens(&mut self, extra: &HashMap<String, String>) -> io::Result<()> {
        for (key, value) in extra.iter() {
            self.0.write_all(b" ")?;
            self.write_escaped_chars(key)?;
            self.0.write_all(b"=")?;
            self.write_escaped_chars(value)?;
        }
        Ok(())
    }

    #[inline]
    fn write_raw(&mut self, s: &str) -> io::Result<()> {
        self.0.write_all(s.as_bytes())
    }

    fn write_escaped_os_str(&mut self, s: &std::ffi::OsStr) -> io::Result<()> {
        for chunk in iter_utf8_chunks(s.as_bytes()) {
            match chunk {
                Utf8Chunk::Valid(chars) => self.write_escaped_chars(chars)?,
                Utf8Chunk::Bytes(bytes) => self.write_escaped_bytes(bytes)?,
            }
        }
        Ok(())
    }

    fn write_escaped_chars(&mut self, mut s: &str) -> io::Result<()> {
        loop {
            if let Some(next) = s.find(|ch| { ch == '\\' || ch == '=' || ch == ' ' || ch == '\t' || ch == '\n' }) {
                self.write_raw(&s[..next])?;

                // Note that we know that this is ASCII character
                match s.as_bytes()[next] {
                    b'\n' => {
                        self.0.write_all(b"\\n")?;
                    }
                    _ => {
                        self.0.write_all(b"\\")?;
                        self.write_raw(&s[next..next+1])?;
                    }
                }
                s = &s[next+1..];
            } else {
                return self.write_raw(s)
            }
        }
    }

    fn write_escaped_bytes(&mut self, s: &[u8]) -> io::Result<()> {
        let mut start = 0;
        let mut next = 0;
        while next < s.len() {
            match s[next] {
                b'\n' => {
                    if start < next {
                        self.0.write_all(&s[start..next])?;
                    }
                    self.0.write_all(b"\\n")?;
                    next += 1;
                    start = next;
                }
                b'\\' | b'=' | b' ' | b'\t' => {
                    if start < next {
                        self.0.write_all(&s[start..next])?;
                    }
                    self.0.write_all(b"\\")?;
                    start = next;
                    next += 1;
                }
                ch if ch.is_ascii() => {
                    next += 1;
                }
                _ => {
                    if start < next {
                        self.0.write_all(&s[start..next])?;
                    }
                    self.0.write_all(b"\\")?;
                    start = next;
                    next += 1;
                }
            }
        }
        if start < next {
            self.0.write_all(&s[start..next])?;
        }
        Ok(())
    }
}

impl<T: io::Write + AtomicSession> AtomicSession for ContentsWriter<T> {
    type AtomicResult = T::AtomicResult;

    #[inline]
    fn commit(self) -> Self::AtomicResult {
        self.0.commit()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::pretty_bytes::*;
    use spectral::prelude::*;
    use std::convert::*;
    use std::ffi::OsStr;

    #[test]
    fn basic() {
        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Dir {
                path: PathBuf::from("/abc"),
            })
        }).pretty())
        .is_equal_to(b"type=dir path=/abc\n".pretty());

        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Sym {
                path: PathBuf::from("/abc"),
                target: PathBuf::from("/def"),
                mtime: UNIX_EPOCH,
                extra: HashMap::new(),
            })
        }).pretty())
        .is_equal_to(b"type=sym path=/abc target=/def mtime=0\n".pretty());
    }

    #[test]
    fn extra_tokens() {
        let mut extra = HashMap::new();
        extra.insert("part".to_string(), "development".to_string());
        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Sym {
                path: PathBuf::from("/libam.so"),
                target: PathBuf::from("libam.so.1"),
                mtime: UNIX_EPOCH,
                extra
            })
        }).pretty())
        .is_equal_to(b"type=sym path=/libam.so target=libam.so.1 mtime=0 part=development\n".pretty());
    }

    #[test]
    fn special_chars() {
        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Dir {
                path: PathBuf::from("/some spaces"),
            })
        }).pretty())
        .is_equal_to(b"type=dir path=/some\\ spaces\n".pretty());

        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Dir {
                path: PathBuf::from("/some\ttabs"),
            })
        }).pretty())
        .is_equal_to(b"type=dir path=/some\\\ttabs\n".pretty());

        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Dir {
                path: PathBuf::from("/multiple\nlines"),
            })
        }).pretty())
        .is_equal_to(b"type=dir path=/multiple\\nlines\n".pretty());

        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Dir {
                path: PathBuf::from("/-=A\\B=-"),
            })
        }).pretty())
        .is_equal_to(b"type=dir path=/-\\=A\\\\B\\=-\n".pretty());
    }

    #[test]
    fn multibyte_chars() {
        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Dir {
                path: PathBuf::from("/multi☠byte"),
            })
        }).pretty())
        .is_equal_to("type=dir path=/multi☠byte\n".as_bytes().pretty());
    }

    #[test]
    fn invalid_utf8() {
        assert_that!(in_memory(|out| {
            out.write_entry(&Entry::Dir {
                path: PathBuf::from(OsStr::from_bytes(b"/bad\x9cbyte")),
            })
        }).pretty())
        .is_equal_to(b"type=dir path=/bad\\\x9cbyte\n".pretty());
    }

    fn in_memory<F>(actions: F) -> Vec<u8>
    where
        F: FnOnce(&mut ContentsWriter<io::Cursor<Vec<u8>>>) -> io::Result<()>,
    {
        let mut writer = ContentsWriter(io::Cursor::new(vec![0; 1024]));
        actions(&mut writer).unwrap();
        let cursor = writer.into_inner();
        let len = usize::try_from(cursor.position()).unwrap();
        let mut buf = cursor.into_inner();
        buf.truncate(len);
        buf
    }
}
