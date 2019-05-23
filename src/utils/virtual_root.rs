use std::borrow::Cow;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::*;

pub use AnyRoot::*;

/// Represents path inside of virtual root.
pub trait RootPath {
    /// Returns real path to path inside of root filesystem. Most of the time expected to be
    /// different from "/".  on Unix systems.
    fn real_root(&self) -> &Path;

    /// Inner representation of path associated with [`real_root`]. Most of the time "/" on Unix.
    fn inner_root(&self) -> &Path {
        Component::RootDir.as_ref()
    }

    /// Resolve inner path to real one that can be used outside of root filesystem.
    fn real_path<'a>(&self, inner: &'a Path) -> io::Result<Cow<'a, Path>> {
        Ok(Cow::from(
            self.real_root()
                .join(self.relative_from_inner(inner)?),
        ))
    }

    /// Try to identify how real path will look inside of root filesystem.
    fn inner_path<'a>(&self, real: &'a Path) -> io::Result<Cow<'a, Path>> {
        Ok(Cow::from(
            self.inner_root().join(self.relative_from_real(real)?),
        ))
    }

    /// Try to get canoncial representation of inner path with all soft links resolved as if our
    /// root filesystem were under [`real_root`].
    ///
    /// # Errors
    ///
    /// Similar to [`std::fs::canoncialize`], but also can fail in case if relative path is outside
    /// of [`inner_path`].
    fn canonicalize_to_real(&self, inner: &Path) -> io::Result<PathBuf> {
        let rel_path = self.relative_from_inner(inner)?;
        let mut result = self.real_root().to_path_buf();
        let mut level = 0;

        for component in rel_path.components() {
            result.push(component);
            level += 1;
            if let Ok(target) = result.read_link() {
                if target.is_absolute() {
                    // "Reset" to root
                    while level > 0 {
                        result.pop();
                        level -= 1;
                    }
                    result.push(self.relative_from_inner(&target)?);
                }
                // I'm lazy to resolving relative alongside with proper "reset" to root
            }
        }

        result.canonicalize()
    }

    fn relative_from_inner<'a>(&self, inner: &'a Path) -> io::Result<&'a Path> {
        inner.strip_prefix(self.inner_root()).map_err(|err| Error::new(ErrorKind::InvalidInput, err))
    }

    fn relative_from_real<'a>(&self, real: &'a Path) -> io::Result<&'a Path> {
        real.strip_prefix(self.real_root()).map_err(|err| Error::new(ErrorKind::InvalidInput, err))
    }
}

#[derive(Debug)]
pub enum AnyRoot {
    RealRoot,
    RootAtBuf(PathBuf),
}

impl RootPath for AnyRoot {
    fn real_root(&self) -> &Path {
        match self {
            AnyRoot::RealRoot => self.inner_root(),
            AnyRoot::RootAtBuf(ref path) => path,
        }
    }

    fn real_path<'a>(&self, inner: &'a Path) -> io::Result<Cow<'a, Path>> {
        Ok(match self {
            AnyRoot::RealRoot => Cow::from(inner),
            AnyRoot::RootAtBuf(ref root) => {
                Cow::from(root.join(self.relative_from_inner(inner)?))
            }
        })
    }

    fn inner_path<'a>(&self, real: &'a Path) -> io::Result<Cow<'a, Path>> {
        Ok(match self {
            AnyRoot::RealRoot => Cow::from(real),
            AnyRoot::RootAtBuf(..) => {
                Cow::from(self.inner_root().join(self.relative_from_real(real)?))
            }
        })
    }
}

pub fn root_at_buf(root: PathBuf) -> impl RootPath {
    debug_assert!(root.is_absolute());
    RootAtBuf(root)
}
