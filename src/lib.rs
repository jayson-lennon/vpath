// Copyright 2024 Jayson Lennon
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//!
//! # Basic Usage
//!
//! ```
//! # use std::path::PathBuf;
//! use vpath::{AbsolutePath, Dirname, Filename, VirtualPath};
//!
//! // Create a new virtual path
//! let mut path = VirtualPath::default();
//!
//! // Push a dir. `Dirname` ensures that we only have a relative path.
//! path.push_dir(Dirname::try_from("data").unwrap());
//! // Push a dir without any checks.
//! path.push_dir_raw("posts");
//!
//! // Add a file. `Filename` ensures that there are no parent directories in the file path.
//! let blog_post = path.with_file(Filename::try_from("first.md").unwrap());
//! assert_eq!(blog_post.to_path_buf(), PathBuf::from("data/posts/first.md"));
//!
//!
//! // ... process your file data here ...
//!
//!
//! // Set the "base directory" of the blog post. You can use this if you want to copy something
//! // from one location to another.
//! let source_dir = AbsolutePath::try_from("/home/blog/source").unwrap();
//! let blog_post = blog_post.with_base(&source_dir);
//! assert_eq!(blog_post.to_path_buf(), PathBuf::from("/home/blog/source/data/posts/first.md"));
//!
//! // Set the "target directory" of the blog post. This is where you would write your output file.
//! let target_dir = AbsolutePath::try_from("/home/blog/output").unwrap();
//! let blog_post = blog_post.with_base(&target_dir);
//! assert_eq!(blog_post.to_path_buf(), PathBuf::from("/home/blog/output/data/posts/first.md"));
//!
//! // Change extension
//! let blog_post = blog_post.with_base(&target_dir).with_extension("html");
//! assert_eq!(blog_post.to_path_buf(), PathBuf::from("/home/blog/output/data/posts/first.html"));
//!
//! // Don't want to expose `data` as a directory in the output
//! let blog_post = blog_post.with_base(&target_dir).strip_prefix("data").unwrap();
//! assert_eq!(blog_post.to_path_buf(), PathBuf::from("/home/blog/output/posts/first.html"));
//! ```
//!
//! `VirtualPath` uses marker types to differentiate between directories and files. This can help
//! prevent problems with pushing directories after you've already composed a file path.
//!
//! ```
//! use vpath::{VirtualPath, DirMarker, FileMarker};
//!
//! // Default starts with a directory.
//! let path: VirtualPath<DirMarker> = VirtualPath::default();
//!
//! // We pushed a directory, so the marker still reflects this.
//! let path: VirtualPath<DirMarker> = path.with_dir_raw("data");
//!
//! // After adding a filename, the marker changes.
//! let mut path: VirtualPath<FileMarker> = path.with_file_raw("index.md");
//!
//! // We now have filename-specific functionality.
//! path.set_extension("html");
//!
//! // ERROR: we cannot push any more directories
//! // path.push_dir_raw("subdir");
//! ```

use std::{
    ffi::OsStr,
    marker::PhantomData,
    path::{Path, PathBuf, StripPrefixError},
};

/// A filename component for a [`VirtualPath`].
///
/// Filename components consist of a single filename and no parent directories.
#[derive(Debug, Clone)]
pub struct Filename {
    name: PathBuf,
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

/// A directory component for a [`VirtualPath`].
///
/// Directory components consist of a directory possibly containing subdirectores. No absolute
/// paths are allowed.
#[derive(Debug, Clone)]
pub struct Dirname {
    name: PathBuf,
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

/// A [`VirtualPath`] marker used to identify the path as a directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirMarker;

/// A [`VirtualPath`] marker used to identify the path as a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileMarker;

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
pub struct AbsolutePath(PathBuf);

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

/// Generates paths with a "base" that can be switched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualPath<M> {
    base: PathBuf,
    path: PathBuf,
    _phantom: PhantomData<M>,
}

impl<M> VirtualPath<M> {
    /// Generate a new `PathBuf` from the current virtual path.
    ///
    /// # Notes
    ///
    /// The path will be relative if `with_base` has not yet been called.
    pub fn to_path_buf(&self) -> PathBuf {
        let mut target = self.base.to_path_buf();
        target.push(self.path.clone());
        target
    }

    /// Changes the "base" of this virtual path.
    #[must_use]
    pub fn with_base(&self, base: &AbsolutePath) -> VirtualPath<M> {
        VirtualPath {
            base: base.0.clone(),
            path: self.path.clone(),
            _phantom: PhantomData,
        }
    }

    /// Returns `true` if this path has a base.
    pub fn has_base(&self) -> bool {
        self.base.components().count() > 0
    }

    /// Returns `Ok(true)` if the path points at an existing entity.
    pub fn try_exists(&self) -> std::io::Result<bool> {
        self.to_path_buf().try_exists()
    }

    /// Returns `true` if the path points at an existing entity.
    pub fn exists(&self) -> bool {
        self.to_path_buf().exists()
    }

    /// Returns the canonical, absolute form of the path with all intermediate components normalized and symbolic links resolved.
    pub fn canonicalize(&self) -> std::io::Result<PathBuf> {
        self.to_path_buf().canonicalize()
    }

    /// Removes the given prefix from the current virtual path.
    ///
    /// The prefix is only removed from the file/directory path component. The "base" path is not
    /// impacted.
    pub fn strip_prefix<P>(self, prefix: P) -> Result<Self, StripPrefixError>
    where
        P: AsRef<Path>,
    {
        let stripped = self.path.strip_prefix(prefix)?;
        Ok(Self {
            base: self.base,
            path: stripped.to_path_buf(),
            _phantom: PhantomData,
        })
    }
}

impl VirtualPath<DirMarker> {
    /// Push another directory onto this path.
    pub fn push_dir_raw<P>(&mut self, dir: P)
    where
        P: Into<PathBuf>,
    {
        self.path.push(dir.into());
    }

    /// Push another directory onto this path.
    pub fn push_dir(&mut self, dir: Dirname) {
        self.path.push(dir.name);
    }

    /// Return this virtual path with the given directory pushed onto it.
    ///
    /// # Notes
    ///
    /// This is provided for convenience. No checks are performed to confirm whether the `dir` is
    /// absolute or not.
    pub fn with_dir_raw<P>(self, dir: P) -> Self
    where
        P: Into<PathBuf>,
    {
        VirtualPath {
            base: self.base,
            path: {
                let mut path = self.path;
                path.push(dir.into());
                path
            },
            _phantom: PhantomData,
        }
    }

    /// Return this virtual path with the given directory pushed onto it.
    pub fn with_dir(self, dir: Dirname) -> Self {
        self.with_dir_raw(dir.name)
    }

    /// Return this virtual path with the given file pushed onto it.
    ///
    /// # Notes
    ///
    /// This is provided for convenience. No checks are performed to confirm whether the `file` is
    /// a file path or has any directory components.
    pub fn with_file_raw<P>(self, file: P) -> VirtualPath<FileMarker>
    where
        P: Into<PathBuf>,
    {
        VirtualPath {
            base: self.base,
            path: {
                let mut path = self.path;
                path.push(file.into());
                path
            },
            _phantom: PhantomData,
        }
    }

    /// Return this virtual path with the given file pushed onto it.
    pub fn with_file(self, file: Filename) -> VirtualPath<FileMarker> {
        self.with_file_raw(file.name)
    }
}

impl VirtualPath<FileMarker> {
    /// Returns the extension of this file, if any.
    pub fn extension(&self) -> Option<&OsStr> {
        self.path.extension()
    }

    /// Returns the file stem.
    pub fn file_stem(&self) -> &OsStr {
        self.path.file_stem().unwrap()
    }

    /// Sets the extension for this file path.
    pub fn set_extension<S: AsRef<OsStr>>(&mut self, extension: S) {
        self.path.set_extension(extension);
    }

    /// Sets the name for this file path.
    pub fn set_file_name<S: AsRef<OsStr>>(&mut self, file_name: S) {
        self.path.set_file_name(file_name)
    }

    /// Returns this file path with an updated `extension`.
    pub fn with_extension<S: AsRef<OsStr>>(mut self, extension: S) -> Self {
        self.path.set_extension(extension);
        self
    }
}

impl Default for VirtualPath<DirMarker> {
    fn default() -> Self {
        Self {
            base: PathBuf::default(),
            path: PathBuf::default(),
            _phantom: PhantomData,
        }
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

    #[test]
    fn pushes_file_with_parent_dir() {
        let path = VirtualPath::default();

        let file = Filename::try_from("parent/index.html").unwrap();

        let path = path.with_file(file);

        assert_eq!(path.to_path_buf(), PathBuf::from("parent/index.html"));
    }

    #[test]
    fn pushes_file() {
        let path = VirtualPath::default();

        let file = Filename::try_from("index.html").unwrap();

        let path = path.with_file(file);

        assert_eq!(path.to_path_buf(), PathBuf::from("index.html"));
    }

    #[test]
    fn pushes_dir() {
        let mut path = VirtualPath::default();

        let dir = Dirname::try_from("output").unwrap();
        path.push_dir(dir);

        assert_eq!(path.to_path_buf(), PathBuf::from("output"));
    }

    #[test]
    fn with_dir_api() {
        let path = VirtualPath::default()
            .with_dir_raw("output")
            .with_dir(Dirname::try_from("img").unwrap());

        assert_eq!(path.to_path_buf(), PathBuf::from("output/img"));
    }

    #[test]
    fn changes_base() {
        let mut path = VirtualPath::default();

        path.push_dir_raw("output");

        let path = path
            .with_file_raw("index.html")
            .with_base(&AbsolutePath::try_from("/home").unwrap());

        assert_eq!(path.to_path_buf(), PathBuf::from("/home/output/index.html"));
    }

    #[test]
    fn check_base_returns_false_when_no_base_has_been_set() {
        let mut path = VirtualPath::default();

        path.push_dir_raw("output");
        let path = path.with_file_raw("index.html");

        assert!(!path.has_base());
    }

    #[test]
    fn check_base_returns_true_when_base_has_been_set() {
        let mut path = VirtualPath::default();

        path.push_dir_raw("output");
        let path = path
            .with_file_raw("index.html")
            .with_base(&AbsolutePath::try_from("/home").unwrap());

        assert!(path.has_base());
    }

    #[test]
    fn returns_file_stem() {
        let path = VirtualPath::default().with_file_raw("index.html");

        assert_eq!(path.file_stem(), OsStr::new("index"));
    }

    #[test]
    fn returns_file_extension() {
        let path = VirtualPath::default().with_file_raw("index.html");

        assert_eq!(path.extension(), Some(OsStr::new("html")));
    }

    #[test]
    fn returns_none_when_no_file_extension_present() {
        let path = VirtualPath::default().with_file_raw("index");

        assert_eq!(path.extension(), None);
    }

    #[test]
    fn sets_extension() {
        let mut path = VirtualPath::default().with_file_raw("index.html");
        path.set_extension("pdf");

        assert_eq!(path.to_path_buf(), PathBuf::from("index.pdf"));
    }

    #[test]
    fn with_extension_api() {
        let path = VirtualPath::default()
            .with_file_raw("index.html")
            .with_extension("pdf");

        assert_eq!(path.to_path_buf(), PathBuf::from("index.pdf"));
    }

    #[test]
    fn sets_file_name() {
        let mut path = VirtualPath::default().with_file_raw("index.html");
        path.set_file_name("root.html");

        assert_eq!(path.to_path_buf(), PathBuf::from("root.html"));
    }

    #[test]
    fn strips_prefix() {
        let mut path = VirtualPath::default();
        path.push_dir_raw("a/b/c");
        let path = path.with_file_raw("test.html").strip_prefix("a").unwrap();

        assert_eq!(path.to_path_buf(), PathBuf::from("b/c/test.html"));
    }
}
