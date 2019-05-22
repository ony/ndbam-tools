use std::fs::*;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::PackageView;
use crate::contents::*;
use crate::utils::virtual_root::*;

impl PackageView {
    pub fn merge(&self, image: &RootPath, root: &RootPath) -> io::Result<()> {
        let mut content = self.content_writer()?;
        let mut walker = WalkDir::new(image.real_root()).into_iter();
        while let Some(node) = walker.next() {
            let node = node?;
            if node.path() == image.real_root() {
                continue  // skip root dir
            }

            let entry = Entry::from_path(node.path(), image)?;
            let real_target = root.real_path(entry.path()).unwrap();
            content.write_entry(&entry)?;
            match entry {
                Entry::Dir { .. } => {
                    if let Ok(metadata) = real_target.symlink_metadata() {
                        assert!(metadata.is_dir(), "TODO: report conflicting types");
                        assert!(
                            metadata.permissions() == node.path().metadata()?.permissions(),
                            "TODO: resolve permissions conflict"
                        )
                    } else {
                        // TODO: use ErrorKind to fail fast if not cross-filesystem case
                        if rename(node.path(), &real_target).is_ok() {
                            // Record moved folder recursively
                            for subnode in WalkDir::new(&real_target) {
                                let subnode = subnode?;
                                if node.path() == entry.path() {
                                    continue; // skip dir we just moved
                                }
                                content.write_entry(&Entry::from_path(subnode.path(), root)?)?;
                            }

                            // No need to dive in
                            walker.skip_current_dir();
                        } else {
                            create_dir(&real_target)?;
                            // TODO: ensure permissions include owner, caps, etc
                            set_permissions(&real_target, node.path().metadata()?.permissions())?;
                        }
                    }
                }

                Entry::File { .. } | Entry::Sym { .. } => {
                    println!("moving {:?} to {:?}", node.path(), real_target);
                    assert!(!real_target.exists(), "TODO: handle file/symlink collisions");
                    rename(node.path(), &real_target)?; // TODO: handle cross-fileystem via copy
                }
            }
        }
        content.commit()?;
        Ok(())
    }
}

impl Entry {
    pub fn from_path(real_path: &Path, root: &RootPath) -> io::Result<Entry> {
        let path = root.inner_path(real_path).unwrap().into_owned();

        let metadata = real_path.symlink_metadata()?;
        if metadata.is_dir() {
            Ok(Entry::Dir { path })
        } else if metadata.is_file() {
            Ok(Entry::File {
                path,
                md5: file_hash(Algorithm::MD5, real_path)?,
                mtime: metadata.modified()?,
                extra: Default::default(),
            })
        } else {
            Ok(Entry::Sym {
                path,
                target: real_path.read_link()?,
                mtime: metadata.modified()?,
                extra: Default::default(),
            })
        }
    }
}
