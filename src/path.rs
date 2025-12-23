use std::path::{Path, PathBuf};

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CanonicalPathBuf(PathBuf);

impl CanonicalPathBuf {
    pub fn try_from_path(path: &Path) -> Result<Self, std::io::Error> {
        if !path.is_absolute() {
            assert!(path.is_absolute(), "path must be absolute");
        }
        Ok(Self(path.canonicalize()?))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for CanonicalPathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}
