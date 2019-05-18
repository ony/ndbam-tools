use super::*;
use std::fs;

steps!(Env => {
    given regex r"^file (.+)$" (PathBuf) |world, path, step| {
        let real_path = world.real_path(&path);
        create_dir_for(&real_path);
        if let Some(content) = step.docstring() {
            fs::write(&real_path, content)
        } else {
            fs::write(&real_path, "dummy")
        }.expect(format!("write to {:?} (original {:?})", &real_path, &path).as_str());
    };

    given regex r"^semi-binary file (.+)$" (PathBuf) |world, path, step| {
        let real_path = world.real_path(&path);
        let content = encode_semi_binary(step.docstring().expect("docstring is mandatory for semi-binary file"));
        create_dir_for(&real_path);
        fs::write(&real_path, content)
            .expect(format!("write to {:?} (original {:?})", &real_path, &path).as_str());
    };

    given regex r"^dir(?:ectory)? (.+)$" (PathBuf) |world, path, _step| {
        fs::create_dir_all(world.real_path(&path)).unwrap();
    };

    given regex r"^symlink (.+) to (.+)$" (PathBuf, PathBuf) |world, path, target, _step| {
        let real_path = world.real_path(&path);
        create_dir_for(&real_path);
        std::os::unix::fs::symlink(&target, &real_path)
            .expect(format!("symlink at {:?} {:?}", &real_path, &path).as_str());
    };
});

fn create_dir_for(path: &Path) {
    path.parent()
        .map(|parent| fs::create_dir_all(parent).unwrap());
}

fn encode_semi_binary(text: &str) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(text.len());
    let mut it = text.as_bytes().iter();
    loop {
        match it.next() {
            None => break,
            Some(b'\\') => {
                match it.next() {
                    Some(b'x') => {
                        let hex_bytes = &it.as_slice()[..2];
                        let hex = from_ascii(hex_bytes).unwrap_or_else(|| {
                            panic!("Unexpected bytes {:02x?} for hex escape in {}", hex_bytes, text);
                        });
                        it.next();
                        it.next();
                        result.push(u8::from_str_radix(hex, 16).expect(&format!("Invalid hex chars in {}", hex)));
                    },
                    Some(b'\\') => result.push(b'\\'),
                    Some(b'n') => result.push(b'\n'),
                    Some(ch) => panic!("Unexpected control character 0x{:0x} followed after '\\' character", ch),
                    None => panic!("Incomplete escape in {}", text),
                }
            },
            Some(ch) => result.push(*ch),
        }
    }
    result
}

fn from_ascii(bytes: &[u8]) -> Option<&str> {
    if bytes.is_ascii() {
        Some(unsafe { std::str::from_utf8_unchecked(&bytes) })
    } else {
        None
    }
}
