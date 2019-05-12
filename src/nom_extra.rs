use nom::IResult;

pub fn map_utf8(i: Vec<u8>) -> Result<String, u32> {
    std::str::from_utf8(&i).map(String::from).map_err(|_| 0)
}

// XXX: take_while/is_a/is_not are broken in nom-4.x for bounded input

pub fn is_a(whitelist: &'static [u8]) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8]> {
    take_while(move |ch: u8| whitelist.contains(&ch))
}

pub fn is_not(blacklist: &'static [u8]) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8]> {
    take_while(move |ch: u8| !blacklist.contains(&ch))
}

pub fn take_while(cond: impl Fn(u8) -> bool) -> impl Fn(&[u8]) -> IResult<&[u8], &[u8]> {
    move |i: &[u8]| {
        let mut n = 0;
        for ch in i {
            if !cond(*ch) { break }
            n += 1;
        }
        let (chunk, rest) = i.split_at(n);
        Ok((rest, chunk))
    }
}
