//! Public data model produced by the read phase.
//!
//! These types are the boundary between the read phase (parsing markdown from
//! disk into structured data) and the build phase (composing navigation,
//! search maps, and HTML output). Keep them owned and serializable so the
//! build phase can consume them without needing filesystem access.

use std::{ffi, ops, path};

/// A single parsed note. Produced by [`crate::notes::Notes::sync`].
#[derive(Clone, Debug)]
pub struct Note {
    pub file_name: FileName,
    pub frontmatter: Frontmatter,
    pub body: Body,
}

/// The source file name, sans extension logic. Used to derive the output slug and the internal link
/// target.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, derive_more::Display)]
pub struct FileName {
    pub inner: String,
}

impl FileName {
    /// Returns the file name without any extension.
    pub fn stem(&self) -> &str {
        self.inner
            .rsplit_once('.')
            .map(|(stem, _ext)| stem)
            .unwrap_or(&self.inner)
    }
}

impl TryFrom<ffi::OsString> for FileName {
    type Error = ffi::OsString;

    fn try_from(value: ffi::OsString) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: value.into_string()?,
        })
    }
}

/// The parsed markdown body. Kept as an `mdast` tree so the build phase can walk it once to extract
/// links, media, and rendered HTML.
#[derive(Clone, Debug)]
pub struct Body {
    pub inner: markdown::mdast::Node,
}

pub use frontmatter::{Frontmatter, Tag};

pub mod frontmatter {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// User-provided metadata parsed from the YAML frontmatter block.
    ///
    /// `public` is intentionally *not* stored here; it is a filter that is applied during the read
    /// phase and never surfaced downstream.
    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct Frontmatter {
        pub title: Option<Title>,
        pub description: Option<Description>,
        pub image: Option<Image>,
        #[serde(default)]
        pub tags: Vec<Tag>,
        pub created: Option<Timestamp>,
        pub modified: Option<Timestamp>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, derive_more::Display)]
    #[serde(transparent)]
    pub struct Description(pub String);

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Image(pub path::PathBuf);

    /// A single tag. Normalized to lower-case and stripped of surrounding whitespace so `#Math` and
    /// `math` collide in navigation.
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Tag(String);

    impl Tag {
        pub fn new(raw: impl AsRef<str>) -> Self {
            Self(raw.as_ref().trim().trim_start_matches('#').to_lowercase())
        }
    }

    impl ops::Deref for Tag {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl From<&str> for Tag {
        fn from(value: &str) -> Self {
            Self::new(value)
        }
    }

    impl From<String> for Tag {
        fn from(value: String) -> Self {
            Self::new(value)
        }
    }

    /// Free-form timestamp. Not parsed into `chrono` on purpose — the site only echoes it back to
    /// the reader.
    #[derive(Clone, Debug, Serialize, Deserialize, derive_more::Display)]
    #[serde(transparent)]
    pub struct Timestamp(pub String);

    #[derive(Clone, Debug, Serialize, Deserialize, derive_more::Display)]
    #[serde(transparent)]
    pub struct Title(pub String);
}
