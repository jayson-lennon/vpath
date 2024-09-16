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

mod absolute;
mod dirname;
mod filename;
mod marker;
mod virtualpath;

pub use absolute::AbsolutePath;
pub use dirname::Dirname;
pub use filename::Filename;
pub use marker::{DirMarker, FileMarker};
pub use virtualpath::VirtualPath;
