use crate::value::NestedProperty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonPointer<'a, 'b>(pub(crate) &'b [NestedProperty<'a>]);

impl<'a, 'b> ToString for JsonPointer<'a, 'b> {
    fn to_string(&self) -> String {
        if self.0.is_empty() {
            "".to_string()
        } else {
            self.0
                .iter()
                .map(NestedProperty::to_pointer_segment_string)
                .collect()
        }
    }
}

impl<'a, 'b> JsonPointer<'a, 'b> {
    pub fn last(&self) -> Option<&NestedProperty<'a>> {
        self.0.last()
    }

    pub fn parent(&self) -> Option<JsonPointer> {
        self.0.split_last().map(|(_, tail)| JsonPointer(tail))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_empty_path_segments() {
        let path = [];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_string(), "".to_string());
        assert!(pointer.parent().is_none());
    }

    #[test]
    fn pointer_one_path_segment() {
        let path = [NestedProperty::Key("foo")];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_string(), "/foo".to_string());
        assert_eq!(pointer.parent().unwrap().to_string(), "".to_string());
    }

    #[test]
    fn pointer_multiple_path_segments() {
        let path = [
            NestedProperty::Key("foo"),
            NestedProperty::Index(0),
            NestedProperty::Key("bar"),
            NestedProperty::Index(1),
        ];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_string(), "/foo/0/bar/1".to_string());
        assert_eq!(
            pointer.parent().unwrap().to_string(),
            "/foo/0/bar".to_string()
        );
    }

    #[test]
    fn pointer_delimits_special_chars() {
        let path = [NestedProperty::Key("a/b"), NestedProperty::Key("m~n")];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_string(), "/a~1b/m~0n".to_string());
        assert_eq!(pointer.parent().unwrap().to_string(), "/a~1b".to_string());
    }

    #[test]
    fn pointer_handles_nested_empty_string_path_segment() {
        let path = [
            NestedProperty::Key("foo"),
            NestedProperty::Index(0),
            NestedProperty::Key(""),
            NestedProperty::Index(1),
        ];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_string(), "/foo/0//1".to_string());
        assert_eq!(pointer.parent().unwrap().to_string(), "/foo/0/".to_string());
    }

    #[test]
    fn pointer_handles_nested_whitespace_path_segment() {
        let path = [
            NestedProperty::Key(" "),
            NestedProperty::Index(0),
            NestedProperty::Key("  "),
            NestedProperty::Index(1),
        ];
        let pointer = JsonPointer(&path);
        assert_eq!(pointer.to_string(), "/ /0/  /1".to_string());
        assert_eq!(pointer.parent().unwrap().to_string(), "/ /0/  ".to_string());
    }
}
