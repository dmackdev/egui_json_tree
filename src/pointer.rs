#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsonPointer<'a, 'b>(pub(crate) &'b [JsonPointerSegment<'a>]);

impl<'a, 'b> ToJsonPointerString for JsonPointer<'a, 'b> {
    fn to_json_pointer_string(&self) -> String {
        self.0
            .iter()
            .map(ToJsonPointerString::to_json_pointer_string)
            .collect()
    }
}

impl<'a, 'b> JsonPointer<'a, 'b> {
    pub fn last(&self) -> Option<&JsonPointerSegment<'a>> {
        self.0.last()
    }

    pub fn parent(&self) -> Option<JsonPointer> {
        self.0.split_last().map(|(_, init)| JsonPointer(init))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonPointerSegment<'a> {
    Key(&'a str),
    Index(usize),
}

impl<'a> ToString for JsonPointerSegment<'a> {
    fn to_string(&self) -> String {
        match self {
            JsonPointerSegment::Key(key) => key.to_string(),
            JsonPointerSegment::Index(idx) => idx.to_string(),
        }
    }
}

impl<'a> ToJsonPointerString for JsonPointerSegment<'a> {
    fn to_json_pointer_string(&self) -> String {
        match self {
            JsonPointerSegment::Key(key) => {
                format!("/{}", key.replace('~', "~0").replace('/', "~1"))
            }
            JsonPointerSegment::Index(idx) => format!("/{}", idx),
        }
    }
}

pub trait ToJsonPointerString {
    fn to_json_pointer_string(&self) -> String;
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
