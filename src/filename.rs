use std::path::{Path, PathBuf};

/// A filename component for a [`VirtualPath`](crate::VirtualPath).
///
/// Filename components consist of a single filename and no parent directories.
#[derive(Debug, Clone)]
pub struct Filename {
    pub(crate) name: PathBuf,
}

/// An error that may occur when constructing a [`Filename`].
#[derive(Debug)]
pub enum FilenameError {
    /// The filename was empty
    Empty,
    /// The filename was absolute
    HasRoot,
}

impl std::fmt::Display for FilenameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "filename cannot be empty"),
            Self::HasRoot => write!(f, "filename cannot be absolute"),
        }
    }
}

impl std::error::Error for FilenameError {}

impl TryFrom<&str> for Filename {
    type Error = FilenameError;

    /// # Errors
    ///
    /// An `Err` will be returned if the filename is empty or has a root.
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        if path.is_empty() {
            return Err(FilenameError::Empty);
        }
        let path = PathBuf::from(path);

        if path.is_absolute() {
            return Err(FilenameError::HasRoot);
        }

        Ok(Filename { name: path })
    }
}

impl TryFrom<PathBuf> for Filename {
    type Error = FilenameError;

    /// # Errors
    ///
    /// An `Err` will be returned if the filename is empty or has a root.
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(&path)
    }
}

impl TryFrom<&PathBuf> for Filename {
    type Error = FilenameError;

    /// # Errors
    ///
    /// An `Err` will be returned if the filename is empty or has a root.
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&Path> for Filename {
    type Error = FilenameError;

    /// # Errors
    ///
    /// An `Err` will be returned if the filename is empty or has a root.
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if path.components().next().is_none() {
            return Err(FilenameError::Empty);
        }

        if path.is_absolute() {
            return Err(FilenameError::HasRoot);
        }

        Ok(Filename {
            name: path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_to_create_filename_with_absolute_path() {
        let file = Filename::try_from("/test");
        assert!(file.is_err());
    }

    #[test]
    fn fail_to_create_filename_with_empty_path() {
        let file = Filename::try_from("");
        assert!(file.is_err());
    }

    #[test]
    fn fail_to_create_filename_with_empty_path_from_pathbuf() {
        let file = Filename::try_from(PathBuf::from(""));
        assert!(file.is_err());
    }
}
