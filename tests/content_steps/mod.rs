use super::*;
use std::fs;

use assert_fs::prelude::*;
use spectral::prelude::*;

steps!(Env => {
    given regex r"^file (.+)$" (PathBuf) |world, ref path, step| {
        let child_path = world.child_path(path);
        if let Some(content) = step.docstring() {
            child_path.write_str(content)
        } else {
            child_path.touch()
        }.expect(format!("write to {:?} (original {:?})", child_path.path(), &path).as_str());
    };

    given regex r"^semi-binary file (.+)$" (PathBuf) |world, ref path, step| {
        let content = encode_semi_binary(step.docstring().expect("docstring is mandatory for semi-binary file"));
        let child_path = world.child_path(path);
        child_path.write_binary(&content)
            .expect(format!("write to {:?} (original {:?})", child_path.path(), path).as_str());
    };

    given regex r"^dir(?:ectory)? (.+)$" (PathBuf) |world, ref path, _step| {
        let child_path = world.child_path(path);
        child_path.create_dir_all()
            .expect(format!("create directory {:?} (original {:?})", child_path.path(), path).as_str());
    };

    // TODO: move to Unix-specific steps
    given regex r"^symlink (.+) to (.+)$" (PathBuf, PathBuf) |world, ref path, ref target, _step| {
        let child_path = world.child_path(path);
        create_dir_for(child_path.path());
        std::os::unix::fs::symlink(target, child_path.path())
            .expect(format!("create symlink at {:?} {:?}", child_path.path(), path).as_str());
    };

    then regex r"^file (.+) exists$" (PathBuf) |world, ref path, step| {
        let child_path = world.child_path(path);
        child_path.assert(predicate::path::is_file());
        if let Some(content) = step.docstring() {
            child_path.assert(predicate::str::similar(content.clone()));
        }
    };

    then regex r"^directory (.+) exists$" (PathBuf) |world, ref path, _step| {
        let child_path = world.child_path(path);
        child_path.assert(predicate::path::is_dir());
    };

    then regex r"^symlink (.+) to (.+) exists$" (PathBuf, PathBuf) |world, ref path, target, _step| {
        let child_path = world.child_path(path);
        child_path.assert(predicate::path::is_symlink());
        assert_that!(fs::read_link(child_path.path()).unwrap())
            .named(&format!("target for symlink {:?}", path))
            .is_equal_to(target);
    };

    then regex r"^no (?:file|dir|directory|symlink) (.+) exists?$" (PathBuf) |world, ref path, _step| {
        world.child_path(path).assert(predicate::path::missing());
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
