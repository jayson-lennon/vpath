use std::{
    ffi::OsStr,
    marker::PhantomData,
    path::{PathBuf, StripPrefixError},
};

use crate::absolute::AbsolutePath;
use crate::dirname::Dirname;
use crate::filename::Filename;
use crate::marker::{DirMarker, FileMarker};

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
        P: Into<PathBuf>,
    {
        let stripped = self.path.strip_prefix(prefix.into())?;
        Ok(Self {
            base: self.base,
            path: stripped.to_path_buf(),
            _phantom: PhantomData,
        })
    }

    // Returns the path without its final component, if there is one.
    pub fn parent(&self) -> Option<PathBuf> {
        self.to_path_buf()
            .parent()
            .map(|parent| parent.to_path_buf())
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
