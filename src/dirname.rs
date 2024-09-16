use std::path::{Path, PathBuf};

/// A directory component for a [`VirtualPath`](crate::VirtualPath).
///
/// Directory components consist of a directory possibly containing subdirectores. No absolute
/// paths are allowed.
#[derive(Debug, Clone)]
pub struct Dirname {
    pub(crate) name: PathBuf,
}

/// An error that may occur when constructing a [`Dirname`].
#[derive(Debug)]
pub enum DirnameError {
    /// No directory name provided
    Empty,
    /// The dirname was absolute
    Absolute,
}

impl std::fmt::Display for DirnameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "dirname cannot be empty"),
            Self::Absolute => write!(f, "dirname cannot be absolute"),
        }
    }
}

impl std::error::Error for DirnameError {}

impl TryFrom<&str> for Dirname {
    type Error = DirnameError;

    /// # Errors
    ///
    /// An `Err` will be returned if the dirname is an absolute path or if no path was provided.
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        if path.is_empty() {
            return Err(DirnameError::Empty);
        }

        let path = PathBuf::from(path);
        Self::try_from(path)
    }
}

impl TryFrom<PathBuf> for Dirname {
    type Error = DirnameError;

    /// # Errors
    ///
    /// An `Err` will be returned if the dirname is an absolute path or if no path was provided.
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(&path)
    }
}

impl TryFrom<&PathBuf> for Dirname {
    type Error = DirnameError;

    /// # Errors
    ///
    /// An `Err` will be returned if the dirname is an absolute path or if no path was provided.
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&Path> for Dirname {
    type Error = DirnameError;

    /// # Errors
    ///
    /// An `Err` will be returned if the dirname is an absolute path or if no path was provided.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if path.components().next().is_none() {
            return Err(DirnameError::Empty);
        }

        (!path.is_absolute())
            .then_some(Dirname {
                name: path.to_path_buf(),
            })
            .ok_or(DirnameError::Absolute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_to_create_dirname_with_empty_path() {
        let dir = Dirname::try_from("");
        assert!(dir.is_err());
    }

    #[test]
    fn fail_to_create_dirname_with_absolute_path() {
        let dir = Dirname::try_from("/test");
        assert!(dir.is_err());
    }
}
