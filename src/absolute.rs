use std::path::{Path, PathBuf};

/// An error that may occur while working with an [`AbsolutePath`].
#[derive(Debug)]
pub struct AbsolutePathError;

impl std::fmt::Display for AbsolutePathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "path must be absolute")
    }
}

impl std::error::Error for AbsolutePathError {}

/// An absolute path.
#[derive(Clone, Debug)]
pub struct AbsolutePath(pub(crate) PathBuf);

impl TryFrom<&str> for AbsolutePath {
    type Error = AbsolutePathError;

    /// # Errors
    ///
    /// An `Err` will be returned if the path is not absolute.
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        let path = PathBuf::from(path);
        Self::try_from(path)
    }
}

impl TryFrom<PathBuf> for AbsolutePath {
    type Error = AbsolutePathError;

    /// # Errors
    ///
    /// An `Err` will be returned if the path is not absolute.
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(&path)
    }
}

impl TryFrom<&PathBuf> for AbsolutePath {
    type Error = AbsolutePathError;

    /// # Errors
    ///
    /// An `Err` will be returned if the path is not absolute.
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&Path> for AbsolutePath {
    type Error = AbsolutePathError;

    /// # Errors
    ///
    /// An `Err` will be returned if the path is not absolute.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        path.is_absolute()
            .then_some(Self(path.to_path_buf()))
            .ok_or(AbsolutePathError)
    }
}

impl AsRef<Path> for AbsolutePath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

#[allow(clippy::from_over_into)]
impl Into<PathBuf> for AbsolutePath {
    fn into(self) -> PathBuf {
        self.0.clone()
    }
}

#[allow(clippy::from_over_into)]
impl Into<PathBuf> for &AbsolutePath {
    fn into(self) -> PathBuf {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_to_create_absolute_path_with_empty_path() {
        let abs = AbsolutePath::try_from("");
        assert!(abs.is_err());
    }

    #[test]
    fn fail_to_create_absolute_path_with_relative_path() {
        let abs = AbsolutePath::try_from("test");
        assert!(abs.is_err());
    }

    #[test]
    fn create_absolute_path_is_ok_when_using_path_containing_a_root() {
        let abs = AbsolutePath::try_from("/test");
        assert!(abs.is_ok());
    }
}
