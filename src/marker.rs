/// A [`VirtualPath`](crate::VirtualPath) marker used to identify the path as a directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirMarker;

/// A [`VirtualPath`](crate::VirtualPath) marker used to identify the path as a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileMarker;
