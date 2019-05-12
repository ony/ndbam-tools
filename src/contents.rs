use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, Duration, UNIX_EPOCH};

use nom::IResult;
use crate::nom_extra::*;

/// Represents NDBAM/VDB contents entry
///
#[derive(Debug, PartialEq)]
pub enum Entry {
    Dir { path: PathBuf },
    File { path: PathBuf, md5: String, mtime: SystemTime, extra: HashMap<String, String> },
    Sym { path: PathBuf, target: PathBuf, extra: HashMap<String, String> },
}

impl Entry {

    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate totems;
    /// # use std::default::Default;
    /// # use std::path::PathBuf;
    /// # use std::time::{UNIX_EPOCH, Duration};
    /// # use ndbam::contents::Entry;
    ///
    /// assert_ok!(Entry::parse(b"type=dir path=/abc"), value == Entry::Dir { path: PathBuf::from("/abc") });
    /// assert_err!(Entry::parse(b"type=unknown"));
    ///
    /// assert_ok!(Entry::parse(b"type=sym path=/def target=abc"), value == Entry::Sym {
    ///            path: PathBuf::from("/def"),
    ///            target: PathBuf::from("abc"),
    ///            extra: Default::default() });
    ///
    /// assert_ok!(Entry::parse(b"type=file path=/abc/f md5=d692bb800 mtime=1549752022"), value == Entry::File {
    ///            path: PathBuf::from("/abc/f"),
    ///            md5: "d692bb800".to_string(),
    ///            mtime: UNIX_EPOCH + Duration::from_secs(1549752022),
    ///            extra: Default::default() });
    /// ```
    pub fn parse(i: &[u8]) -> Result<Entry, String> {
        let (rest, mut fields) = stringify_err(tokens(i))?;
        debug_assert!(rest.is_empty(), format!("Unexpected trailing input: {:02x?}", rest));

        let kind = fields.try_take("type")?;
        let path = PathBuf::from(fields.try_take("path")?);

        match kind.as_str() {
            "file" => Ok(Entry::File { path,
                md5: fields.try_take("md5")?,
                mtime: {
                    let secs = stringify_err(fields.try_take("mtime")?.parse::<u64>())?;
                    UNIX_EPOCH + Duration::from_secs(secs)
                },
                extra: fields.take_extra()}),

            "dir" => fields.no_extra_for(Entry::Dir { path }),

            "sym" => Ok(Entry::Sym { path,
                target: PathBuf::from(fields.try_take("target")?),
                extra: fields.take_extra()}),

            _ => Err(format!("Unknown type {:?}", kind))
        }
    }
}

fn stringify_err<T, E: std::string::ToString>(res: Result<T, E>) -> Result<T, String> {
    res.map_err(|err| err.to_string())
}

trait TokensExt<E> {
    fn try_take(&mut self, key: &str) -> Result<String, E>;
    fn take_extra(&mut self) -> HashMap<String, String>;
    fn no_extra_for<T: std::fmt::Debug>(&self, val: T) -> Result<T, E>;
}

impl<'s> TokensExt<String> for Tokens<'s> {
    fn try_take(&mut self, key: &str) -> Result<String, String> {
        self.remove(key).map_or_else(|| Err(format!("Missing {:?} in {:?}", key, self)), Ok)
    }

    fn take_extra(&mut self) -> HashMap<String, String> {
        let mut extra = HashMap::new();
        for (k, v) in self.drain() {
            extra.insert(k.to_string(), v);
        }
        extra
    }

    fn no_extra_for<T: std::fmt::Debug>(&self, val: T) -> Result<T, String> {
        if !self.is_empty() {
            Err(format!("Unexpected fields {:?} for {:?}", self.keys(), val))
        } else {
            Ok(val)
        }
    }
}

// TODO: consider using 'OsString' for values
type Token<'s> = (&'s str, String);
type Tokens<'s> = HashMap<&'s str, String>;

named!(hspace<&[u8], ()>, do_parse!(verify!(call!(is_a(b" \t")), |sp: &[u8]| sp.len() > 0) >> ()));
named!(unescaped_value_chunk<&[u8], &[u8]>, verify!(call!(is_not(b" \t\\")), |chunk: &[u8]| chunk.len() > 0));
named!(key<&[u8], &str>, do_parse!(name: is_not!(b"=") >> (std::str::from_utf8(name).unwrap())));
named!(value<&[u8], String>, map_res!(
    escaped_transform!(unescaped_value_chunk, b'\\', alt!(tag!("n") => { |_| &b"\n"[..] } | take!(1))),
    map_utf8));

named!(token<&[u8], Token>, separated_pair!(key, char!('='), value));

fn tokens(i: &[u8]) -> IResult<&[u8], Tokens> {
    let mut rest = i;
    let mut tokens = HashMap::new();
    loop {
        let (rest_, tok) = token(rest)?;
        if let Some(_) = tokens.insert(tok.0, tok.1) {
            fail(rest_)?;  // TODO: error details
        }
        if let Ok((rest__, _)) = hspace(rest_) {
            rest = rest__;
        } else {
            return Ok((rest_, tokens));
        }
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test] fn key_basic() {
        assert_eq!(key(b"abc=def"), Ok((&b"=def"[..], "abc")));
    }

    #[test] fn key_bad() {
        assert!(key(b"=def").is_err());
        assert!(key(b"").is_err());
    }

    #[test] fn value_basic() {
        assert_ok!(value(b"hi there"), value == (&b" there"[..], "hi".to_string()));
        assert_ok!(value(b"hi"), value == (&b""[..], "hi".to_string()));
    }

    #[test] fn value_bad() {
        assert!(value(b"F\\").is_err());
    }

    #[test] fn value_escapes() {
        assert_eq!(value(b"\\  "), Ok((&b" "[..], " ".to_string())));
        assert_eq!(value(b"\\\\ "), Ok((&b" "[..], "\\".to_string())));
        assert_eq!(value(b"\\n "), Ok((&b" "[..], "\n".to_string())));
    }

    #[test] fn value_paludis_escapes() {
        assert_eq!(value(b"_\\=Class_Gold\\=_ "), Ok((&b" "[..], "_=Class_Gold=_".to_string())));
        assert_eq!(value(b"\\t "), Ok((&b" "[..], "t".to_string())));
    }

    #[test] fn value_paludis_utf8() {
        assert_ok!(value(b"F\\\xc5\\\x91tan\\\xc3\\\xbas\\\xc3\xadtv\\\xc3\\\xa1ny.crt "), value == (&b" "[..], "Főtanúsítvány.crt".to_string()));
    }

    #[test] fn value_bad_utf8() {
        assert_err!(value(b"F\\\xc5\\\xc3\\\x91tan"));
    }

    #[test] fn token_basic() {
        assert_ok!(token(b"abc=def "), value == (&b" "[..], ("abc", "def".to_string())));
    }

    #[test] fn token_bad() {
        assert_err!(token(b"abc "));
        assert_err!(token(b""));
    }

    #[test] fn tokens_basic() {
        assert_ok!(tokens(b"abc=def"));
    }

    #[test] fn tokens_multiple() {
        assert_ok!(tokens(b"abc=def ghi=jkl").map(|(_, h)| h.len()), value >= 2);
    }

    #[test] fn tokens_bad() {
        assert_err!(tokens(b""));
        assert_err!(tokens(b" "));
        assert_err!(tokens(b"="));
        assert_err!(tokens(b"a=x a=y"));
    }
}
