use std::borrow::Cow;
use std::io;
use std::path::*;

pub use AnyRoot::*;

pub trait RootPath {
    fn real_root(&self) -> &Path;

    fn inner_root(&self) -> &Path {
        Component::RootDir.as_ref()
    }

    fn real_path<'a>(&self, inner: &'a Path) -> Result<Cow<'a, Path>, StripPrefixError> {
        Ok(Cow::from(
            self.real_root()
                .join(inner.strip_prefix(self.inner_root())?),
        ))
    }
    fn inner_path<'a>(&self, real: &'a Path) -> Result<Cow<'a, Path>, StripPrefixError> {
        Ok(Cow::from(
            self.inner_root().join(real.strip_prefix(self.real_root())?),
        ))
    }

    fn canonicalize_to_real(&self, target: &Path) -> io::Result<PathBuf> {
        debug_assert!(target.is_absolute());
        let mut result = self.real_root().to_path_buf();
        let mut level = 0;
        let mut components = target.components();
        components.next(); // skip leading root indicator
        for component in components {
            result.push(component);
            level += 1;
            if let Ok(target) = result.read_link() {
                if target.is_absolute() {
                    // "Reset" to root
                    while level > 0 {
                        result.pop();
                        level -= 1;
                    }
                    result.push(target);
                }
                // I'm lazy to resolving relative alongside with proper "reset" to root
            }
        }

        result.canonicalize()
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

    fn real_path<'a>(&self, inner: &'a Path) -> Result<Cow<'a, Path>, StripPrefixError> {
        Ok(match self {
            AnyRoot::RealRoot => Cow::from(inner),
            AnyRoot::RootAtBuf(ref path) => {
                Cow::from(path.join(inner.strip_prefix(self.inner_root())?))
            }
        })
    }

    fn inner_path<'a>(&self, real: &'a Path) -> Result<Cow<'a, Path>, StripPrefixError> {
        Ok(match self {
            AnyRoot::RealRoot => Cow::from(real),
            AnyRoot::RootAtBuf(ref path) => {
                Cow::from(self.inner_root().join(real.strip_prefix(path)?))
            }
        })
    }
}

pub fn root_at_buf(root: PathBuf) -> impl RootPath {
    debug_assert!(root.is_absolute());
    RootAtBuf(root)
}
