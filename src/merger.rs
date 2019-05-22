use std::fs::*;
use std::io;
use std::path::Path;
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
                continue; // skip root dir
            }

            let entry = Entry::from_path(node.path(), image)?;
            let merged_path = root.real_path(entry.path()).unwrap();
            content.write_entry(&entry)?;
            match entry {
                Entry::Dir { .. } => {
                    if let Ok(metadata) = merged_path.symlink_metadata() {
                        assert!(metadata.is_dir(), "TODO: report conflicting types");
                        assert!(
                            metadata.permissions() == node.path().metadata()?.permissions(),
                            "TODO: resolve permissions conflict"
                        )
                    } else {
                        // TODO: use ErrorKind to fail fast if not cross-filesystem case
                        if rename(node.path(), &merged_path).is_ok() {
                            // Record moved folder recursively
                            for subnode in WalkDir::new(&merged_path) {
                                let subnode = subnode?;
                                if node.path() == entry.path() {
                                    continue; // skip dir we just moved
                                }
                                content.write_entry(&Entry::from_path(subnode.path(), root)?)?;
                            }

                            // No need to dive in
                            walker.skip_current_dir();
                        } else {
                            create_dir(&merged_path)?;
                            // TODO: ensure permissions include owner, caps, etc
                            set_permissions(&merged_path, node.path().metadata()?.permissions())?;
                        }
                    }
                }

                Entry::File { .. } | Entry::Sym { .. } => {
                    if let Entry::Sym { path, target, .. } = &entry {
                        let target = if target.is_absolute() {
                            target.to_owned()
                        } else {
                            // TODO: ran-away link check
                            path.parent().expect("Root cannot be symlink").join(target)
                        };

                        if root.canonicalize_to_real(&target).is_ok() {
                            // Looks good. We point to something that exist in filesystem where we
                            // plan to install. Now just ensure it will not be deleted during
                            // further merge. I.e. ensure that we are not pointing into image
                            // itself.
                            let merged_target = root.real_path(&target).unwrap();
                            assert!(
                                image.inner_path(&merged_target).is_err(),
                                "Symlink target should not point back into image"
                            );
                        } else {
                            // Probably we didn't installed path that symlink is pointing to. Let's
                            // check if it exists in the image itself.
                            image
                                .canonicalize_to_real(&target)
                                .expect("Symlink pointing to some existing object");
                        }
                    }
                    println!("moving {:?} to {:?}", node.path(), merged_path);
                    assert!(
                        !merged_path.exists(),
                        "TODO: handle file/symlink collisions"
                    );
                    rename(node.path(), &merged_path)?; // TODO: handle cross-fileystem via copy
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
