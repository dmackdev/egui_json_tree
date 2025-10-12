//! A JSON Pointer implementation for identifying specific values within a JSON document.

use std::fmt;

/// A JSON Pointer implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonPointer<'a, 'b>(pub(crate) &'b [JsonPointerSegment<'a>]);

impl<'a> JsonPointer<'a, '_> {
    /// Returns a JSON Pointer string that can be used to look up specific values within a JSON document, where:
    /// - The whole document is identified by the empty string `""`.
    /// - A pointer string to a value within the document starts with `/`.
    /// - The pointer string is comprised of segments separated by `/`.
    /// - Each segment represents either an array index or object key.
    /// - The special character `~` in an object key is delimited as `~0`.
    /// - The special character `/` in an object key is delimited as `~1`.
    pub fn to_json_pointer_string(&self) -> String {
        self.0
            .iter()
            .map(JsonPointerSegment::to_json_pointer_segment_string)
            .collect()
    }

    /// Returns the last [JsonPointerSegment] of this pointer, if it exists.
    ///
    /// This is useful for retrieving the array index or object key that points to a JSON value.
    pub fn last(&self) -> Option<&JsonPointerSegment<'a>> {
        self.0.last()
    }

    /// Returns a [JsonPointer] to the parent of this pointer, if it exists.
    ///
    /// This is useful for retrieving a pointer to the enclosing array or object of a JSON value.
    pub fn parent(&self) -> Option<JsonPointer> {
        self.0.split_last().map(|(_, init)| JsonPointer(init))
    }
}

/// An individual segment of a [JsonPointer] - either an array index or object key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonPointerSegment<'a> {
    Index(usize),
    Key(&'a str),
}

impl fmt::Display for JsonPointerSegment<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonPointerSegment::Key(key) => write!(f, "{key}"),
            JsonPointerSegment::Index(idx) => write!(f, "{idx}"),
        }
    }
}

impl JsonPointerSegment<'_> {
    pub fn to_json_pointer_segment_string(&self) -> String {
        match self {
            JsonPointerSegment::Key(key) => {
                format!("/{}", key.replace('~', "~0").replace('/', "~1"))
            }
            JsonPointerSegment::Index(idx) => format!("/{idx}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_empty_path_segments() {
        let path = [];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_json_pointer_string(), "".to_string());
        assert!(pointer.parent().is_none());
    }

    #[test]
    fn pointer_one_path_segment() {
        let path = [JsonPointerSegment::Key("foo")];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_json_pointer_string(), "/foo".to_string());
        assert_eq!(
            pointer.parent().unwrap().to_json_pointer_string(),
            "".to_string()
        );
    }

    #[test]
    fn pointer_multiple_path_segments() {
        let path = [
            JsonPointerSegment::Key("foo"),
            JsonPointerSegment::Index(0),
            JsonPointerSegment::Key("bar"),
            JsonPointerSegment::Index(1),
        ];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_json_pointer_string(), "/foo/0/bar/1".to_string());
        assert_eq!(
            pointer.parent().unwrap().to_json_pointer_string(),
            "/foo/0/bar".to_string()
        );
    }

    #[test]
    fn pointer_delimits_special_chars() {
        let path = [
            JsonPointerSegment::Key("a/b"),
            JsonPointerSegment::Key("m~n"),
        ];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_json_pointer_string(), "/a~1b/m~0n".to_string());
        assert_eq!(
            pointer.parent().unwrap().to_json_pointer_string(),
            "/a~1b".to_string()
        );
    }

    #[test]
    fn pointer_handles_nested_empty_string_path_segment() {
        let path = [
            JsonPointerSegment::Key("foo"),
            JsonPointerSegment::Index(0),
            JsonPointerSegment::Key(""),
            JsonPointerSegment::Index(1),
        ];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_json_pointer_string(), "/foo/0//1".to_string());
        assert_eq!(
            pointer.parent().unwrap().to_json_pointer_string(),
            "/foo/0/".to_string()
        );
    }

    #[test]
    fn pointer_handles_nested_whitespace_path_segment() {
        let path = [
            JsonPointerSegment::Key(" "),
            JsonPointerSegment::Index(0),
            JsonPointerSegment::Key("  "),
            JsonPointerSegment::Index(1),
        ];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_json_pointer_string(), "/ /0/  /1".to_string());
        assert_eq!(
            pointer.parent().unwrap().to_json_pointer_string(),
            "/ /0/  ".to_string()
        );
    }
}
