pub mod contents;
pub mod merger;
mod utils;

use std::path::{Path, PathBuf};
use std::fs::ReadDir;
use std::io::prelude::*;
use std::process;
use std::time::UNIX_EPOCH;
use std::{fs, io};
pub use utils::virtual_root::*;

pub struct NDBAM<'p> {
    location: &'p Path,
}

impl<'p> NDBAM<'p> {
    pub fn new(location: &Path) -> NDBAM {
        let mut sub = location.to_path_buf();
        sub.push("ndbam.conf");
        assert!(sub.is_file(), "Only existing ndbam repositories supported at this moment");
        // TODO: check ndbam_format == 1
        // TODO: consider repostiroy_format for specific content
        // TODO: create new if absent
        NDBAM { location: location }
    }

    pub fn versions_of(&self, name: &str) -> Option<impl Iterator<Item=PackageView>> {
        if let Ok(versions) = self.versions_path(name).read_dir() {
            Some(PackageVersionsIter { versions })
        } else {
            None
        }
    }

    pub fn all_packages(&self) -> Option<impl Iterator<Item=PackageView>> {
        let mut names = self.location.join("data").read_dir().expect("broken layout");
        AllPackagesIter::next_versions(&mut names).map(|versions| {
            AllPackagesIter {
                names: names,
                versions: versions,
            }
        })
    }

    pub fn new_package_version(&self, name: &str, version: &str, slot: &str) -> PackageView {
        let location = self.versions_path(name).join(format!("{}:{}:{}", version, slot, magic_cookie()));
        fs::create_dir_all(&location).unwrap();
        PackageView { location }
    }

    fn versions_path(&self, name: &str) -> PathBuf {
        self.location.join("data").join(name.replace("/", "---"))
    }
}

struct AllPackagesIter {
    names: ReadDir,
    versions: PackageVersionsIter,
}

impl AllPackagesIter {
    fn next_versions(names: &mut ReadDir) -> Option<PackageVersionsIter> {
        while let Some(name) = names.next().map(Result::unwrap) {
            if let Ok(versions) = name.path().read_dir() {
                return Some(PackageVersionsIter { versions });
            } else {
                assert!(!name.file_type().unwrap().is_dir())
            }
        }
        None
    }
}

impl Iterator for AllPackagesIter {
    type Item = PackageView;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(version) = self.versions.next() {
                return Some(version)
            } else if let Some(versions) = AllPackagesIter::next_versions(&mut self.names) {
                self.versions = versions
            } else {
                return None
            }
        }
    }
}

struct PackageVersionsIter {
    versions: ReadDir,
}

impl Iterator for PackageVersionsIter {
    type Item = PackageView;

    fn next(&mut self) -> Option<Self::Item> {
        self.versions.next()
            .map(|version| PackageView { location: version.unwrap().path() })
    }
}

pub struct PackageView {
    location: PathBuf,
}

impl PackageView {
    pub fn name(&self) -> String {
        self.location.parent().unwrap().file_name().unwrap().to_str().unwrap().replace("---", "/")
    }

    pub fn version(&self) -> &str {
        self.location.file_name().unwrap().to_str().unwrap().split(':').next().unwrap()
    }

    pub fn slot(&self) -> Option<&str> {
        let mut tokens = self.location.file_name().unwrap().to_str().unwrap().split(':');
        tokens.next().unwrap();
        tokens.next()
    }

    pub fn full_name(&self) -> String {
        format!("{}-{}", self.name(), self.version())
    }

    pub fn read_key(&self, key: &str) -> io::Result<String> {
        std::fs::read_to_string(self.location.join(key))
    }

    pub fn contents(&self) -> impl Iterator<Item=contents::Entry> {
        let f = std::fs::File::open(self.location.join("contents")).expect("invalid package entry");
        io::BufReader::new(f)
            .split(b'\n')
            .map(|row| contents::Entry::parse(&row.unwrap()).unwrap())
    }

    pub fn content_writer(&self) -> io::Result<contents::ContentsWriter> {
        contents::create(self.location.join("contents"))
    }
}

/// Generates pseudo-unique string suitable for using in filenames.
pub fn magic_cookie() -> String {
    let epoch = UNIX_EPOCH.elapsed().unwrap();

    format!("C.{}.{}.{}.C",
            process::id(),
            epoch.as_secs(),
            epoch.subsec_micros())
}
