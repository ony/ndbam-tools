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
    File { path: PathBuf, md5: String, mtime: SystemTime },
    Sym { path: PathBuf, target: PathBuf },
}

impl Entry {

    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate totems;
    /// # use std::path::PathBuf;
    /// # use std::time::{UNIX_EPOCH, Duration};
    /// # use ndbam::contents::Entry;
    ///
    /// assert_eq!(Entry::parse(b"type=dir path=/abc"), Ok(Entry::Dir { path: PathBuf::from("/abc") }));
    /// assert_eq!(Entry::parse(b"type=sym path=/def target=abc"), Ok(Entry::Sym { path: PathBuf::from("/def"), target: PathBuf::from("abc") }));
    /// assert_eq!(Entry::parse(b"type=file path=/abc/f md5=d692bb800 mtime=1549752022"),
    ///            Ok(Entry::File { path: PathBuf::from("/abc/f"), md5: "d692bb800".to_string(), mtime: UNIX_EPOCH + Duration::from_secs(1549752022) }));
    /// assert_err!(Entry::parse(b"type=unknown"));
    /// assert_err!(Entry::parse(b"="));
    /// ```
    pub fn parse(i: &[u8]) -> Result<Entry, String> {
        fn stringify_err<T, E: std::string::ToString>(res: Result<T, E>) -> Result<T, String> {
            res.map_err(|err| err.to_string())
        }

        let (rest, mut fields) = stringify_err(tokens(i))?;
        if !rest.is_empty() {
            return Err(format!("Trailing input: {:02x?}", rest))
        }

        let mut field = |key: &str| {
            fields.remove(key).map_or_else(|| Err(format!("Missing {:?} in {:?}", key, fields)), Ok)
        };

        let kind: String = field("type")?;
        let path = PathBuf::from(field("path")?);

        match kind.as_str() {
            "file" => Ok(Entry::File { path, md5: field("md5")?, mtime: {
                let secs = stringify_err(field("mtime")?.parse::<u64>())?;
                UNIX_EPOCH + Duration::from_secs(secs)
            }}),
            "dir" => Ok(Entry::Dir { path }),
            "sym" => Ok(Entry::Sym { path, target: PathBuf::from(field("target")?) }),
            _ => Err(format!("Unknown type {:?}", kind))
        }
    }
}

// TODO: consider using 'OsString' for values
type Token<'s> = (&'s str, String);
type Tokens<'s> = HashMap<&'s str, String>;

named!(hspace<&[u8], ()>, do_parse!(verify!(call!(is_a(b" \t")), |sp: &[u8]| sp.len() > 0) >> ()));
named!(unescaped_value_chunk<&[u8], &[u8]>, verify!(call!(is_not(b" \t\\")), |chunk: &[u8]| chunk.len() > 0));
named!(key<&[u8], &str>, do_parse!(name: is_not!(b"=") >> (std::str::from_utf8(name).unwrap())));
named!(value<&[u8], String>, map_res_err!(
    escaped_transform!(unescaped_value_chunk, b'\\', alt!(tag!("n") => { |_| &b"\n"[..] } | take!(1))),
    map_utf8));

named!(token<&[u8], Token>, separated_pair!(key, char!('='), value));

fn tokens(i: &[u8]) -> IResult<&[u8], Tokens> {
    let mut rest = i;
    let mut tokens = HashMap::new();
    loop {
        let (rest_, token) = token(rest)?;
        tokens.insert(token.0, token.1);
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
        assert_ok!(tokens(b"abc=def ghi=jkl").map(|(r, h)| h.len()), value >= 2);
    }

    #[test] fn tokens_bad() {
        assert_err!(tokens(b""));
        assert_err!(tokens(b" "));
        assert_err!(tokens(b"="));
    }

}
