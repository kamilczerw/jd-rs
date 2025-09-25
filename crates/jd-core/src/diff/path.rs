use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Represents a single element within a diff path.
///
/// A segment can either refer to an object key or an array index. Future
/// milestones will add set and multiset markers.
///
/// ```
/// # use jd_core::diff::PathSegment;
/// let key = PathSegment::key("name");
/// let index = PathSegment::index(2);
/// assert!(matches!(key, PathSegment::Key(_)));
/// assert!(matches!(index, PathSegment::Index(_)));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PathSegment {
    /// Object key lookup.
    Key(String),
    /// Array index lookup.
    Index(i64),
}

impl PathSegment {
    /// Creates a key segment.
    #[must_use]
    pub fn key<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self::Key(value.into())
    }

    /// Creates an index segment.
    #[must_use]
    pub fn index<I>(value: I) -> Self
    where
        I: Into<i64>,
    {
        Self::Index(value.into())
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(key) => f.write_str(key),
            Self::Index(index) => write!(f, "{index}"),
        }
    }
}

impl Serialize for PathSegment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Key(key) => serializer.serialize_str(key),
            Self::Index(index) => serializer.serialize_i64(*index),
        }
    }
}

impl<'de> Deserialize<'de> for PathSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = PathSegment;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a string key or integer index")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PathSegment::Key(v.to_owned()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PathSegment::Key(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PathSegment::Index(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let value = i64::try_from(v).map_err(|_| E::custom("index exceeds i64"))?;
                Ok(PathSegment::Index(value))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Represents the fully qualified location of a diff hunk within a document.
///
/// ```
/// # use jd_core::diff::{Path, PathSegment};
/// let path = Path::new().with_segment(PathSegment::key("foo"))
///     .with_segment(PathSegment::index(0));
/// assert_eq!(path.len(), 2);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Path(Vec<PathSegment>);

impl Path {
    /// Creates an empty path.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a new segment, returning the extended path.
    #[must_use]
    pub fn with_segment(mut self, segment: PathSegment) -> Self {
        self.0.push(segment);
        self
    }

    /// Returns the underlying segments.
    #[must_use]
    pub fn segments(&self) -> &[PathSegment] {
        &self.0
    }

    /// Returns the number of segments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Indicates whether the path is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a new path with the last segment removed, if any.
    ///
    /// ```
    /// # use jd_core::diff::{Path, PathSegment};
    /// let path = Path::new().with_segment(PathSegment::index(1));
    /// assert!(path.drop_last().is_empty());
    /// ```
    #[must_use]
    pub fn drop_last(&self) -> Self {
        let mut segments = self.0.clone();
        segments.pop();
        Self(segments)
    }

    /// Consumes the path and returns the owned segments.
    ///
    /// ```
    /// # use jd_core::diff::{Path, PathSegment};
    /// let path = Path::from(PathSegment::key("id"));
    /// let segments = path.into_segments();
    /// assert_eq!(segments.len(), 1);
    /// ```
    #[must_use]
    pub fn into_segments(self) -> Vec<PathSegment> {
        self.0
    }

    /// Pushes a new segment in-place.
    ///
    /// ```
    /// # use jd_core::diff::{Path, PathSegment};
    /// let mut path = Path::new();
    /// path.push(PathSegment::key("name"));
    /// assert_eq!(path.len(), 1);
    /// ```
    pub fn push(&mut self, segment: PathSegment) {
        self.0.push(segment);
    }

    /// Pops the last segment off the path.
    ///
    /// ```
    /// # use jd_core::diff::{Path, PathSegment};
    /// let mut path = Path::from(PathSegment::index(0));
    /// assert!(path.pop().is_some());
    /// assert!(path.is_empty());
    /// ```
    pub fn pop(&mut self) -> Option<PathSegment> {
        self.0.pop()
    }
}

impl From<Vec<PathSegment>> for Path {
    fn from(value: Vec<PathSegment>) -> Self {
        Self(value)
    }
}

impl From<PathSegment> for Path {
    fn from(value: PathSegment) -> Self {
        Self(vec![value])
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (idx, segment) in self.0.iter().enumerate() {
            if idx > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{segment}")?;
        }
        f.write_str("]")
    }
}

impl<'a> IntoIterator for &'a Path {
    type Item = &'a PathSegment;
    type IntoIter = std::slice::Iter<'a, PathSegment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for Path {
    type Item = PathSegment;
    type IntoIter = std::vec::IntoIter<PathSegment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Creates a path representing the root of a document.
///
/// ```
/// # use jd_core::diff::root_path;
/// let path = root_path();
/// assert!(path.is_empty());
/// ```
#[must_use]
pub fn root_path() -> Path {
    Path::new()
}

/// Builds a path from an iterator of segments.
///
/// ```
/// # use jd_core::diff::{path_from_segments, PathSegment};
/// let path = path_from_segments([PathSegment::key("a"), PathSegment::index(1)]);
/// assert_eq!(path.len(), 2);
/// ```
#[must_use]
pub fn path_from_segments<I>(segments: I) -> Path
where
    I: IntoIterator<Item = PathSegment>,
{
    Path(segments.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip_for_key_segments() {
        let path = path_from_segments([PathSegment::key("foo"), PathSegment::index(3)]);
        let json = serde_json::to_string(&path).unwrap();
        assert_eq!(json, "[\"foo\",3]");
        let decoded: Path = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, path);
    }
}
