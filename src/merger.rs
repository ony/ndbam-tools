use std::fs::*;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::PackageView;
use crate::contents::*;

impl PackageView {
    pub fn merge(&self, image: &Path, root: &Path) -> io::Result<()> {
        let mut content = self.content_writer()?;
        let mut walker = WalkDir::new(image).into_iter();
        while let Some(node) = walker.next() {
            let node = node?;
            if node.path() == image {
                continue  // skip root dir
            }

            let entry = Entry::from_path(node.path(), image)?;
            let real_target = entry.path_in(root);
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
    pub fn from_path(real_path: &Path, root: &Path) -> io::Result<Entry> {
        let mut path = PathBuf::from("/");
        path.push(real_path.strip_prefix(root).unwrap());

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
