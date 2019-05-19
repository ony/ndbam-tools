use std::str::*;
use Utf8Chunk::*;

#[derive(PartialEq, Debug)]
pub enum Utf8Chunk<'s> {
    Valid(&'s str),
    Bytes(&'s [u8]),
}

pub struct Utf8ChunksIter<'s> {
    bytes: &'s [u8],
    err: Option<Utf8Error>,
}

pub fn iter_utf8_chunks(bytes: &[u8]) -> Utf8ChunksIter {
    Utf8ChunksIter { bytes, err: None }
}

impl<'s> Utf8ChunksIter<'s> {
    #[cfg(test)]
    pub fn into_vec(self) -> Vec<Utf8Chunk<'s>> {
        let mut chunks = Vec::new();
        for chunk in self {
            chunks.push(chunk);
        }
        chunks
    }
}

impl<'s> Iterator for Utf8ChunksIter<'s> {
    type Item = Utf8Chunk<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.err {
            self.err = None;
            self.bytes = &self.bytes[err.valid_up_to()..];
            return if let Some(len) = err.error_len() {
                let (bytes, tail) = self.bytes.split_at(len);
                self.bytes = tail;
                Some(Bytes(bytes))
            } else {
                None
            };
        }

        match from_utf8(self.bytes) {
            Ok(chars) => {
                if chars.is_empty() {
                    None
                } else {
                    self.bytes = b"";
                    Some(Valid(chars))
                }
            }
            Err(err) => {
                self.err = Some(err);
                Some(Valid(unsafe {
                    from_utf8_unchecked(&self.bytes[..err.valid_up_to()])
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    pub fn ascii() {
        assert_that!(iter_utf8_chunks(b"").into_vec()).is_empty();
        assert_that!(iter_utf8_chunks(b"abc").into_vec()).is_equal_to(vec![Valid("abc")]);
    }

    #[test]
    pub fn multibyte_utf8() {
        assert_that!(iter_utf8_chunks(b".o(\xd1\x97\xd0\xb6\xd0\xb0)").into_vec())
            .is_equal_to(vec![Valid(".o(їжа)")]);
    }

    #[test]
    pub fn invalid_utf8() {
        assert_that!(iter_utf8_chunks(b"abc\xc9def").into_vec()).is_equal_to(vec![
            Valid("abc"),
            Bytes(b"\xc9"),
            Valid("def"),
        ]);
    }
}
