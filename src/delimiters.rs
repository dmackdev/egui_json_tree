pub(crate) struct Delimiters {
    pub(crate) collapsed: ExpandableDelimiter,
    pub(crate) collapsed_empty: ExpandableDelimiter,
    pub(crate) opening: ExpandableDelimiter,
    pub(crate) closing: ExpandableDelimiter,
}

pub(crate) const ARRAY_DELIMITERS: Delimiters = Delimiters {
    collapsed: ExpandableDelimiter::CollapsedArray,
    collapsed_empty: ExpandableDelimiter::CollapsedEmptyArray,
    opening: ExpandableDelimiter::OpeningArray,
    closing: ExpandableDelimiter::ClosingArray,
};

pub(crate) const OBJECT_DELIMITERS: Delimiters = Delimiters {
    collapsed: ExpandableDelimiter::CollapsedObject,
    collapsed_empty: ExpandableDelimiter::CollapsedEmptyObject,
    opening: ExpandableDelimiter::OpeningObject,
    closing: ExpandableDelimiter::ClosingObject,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpacingDelimiter {
    Empty,
    Comma,
    Colon,
}

impl AsRef<str> for SpacingDelimiter {
    fn as_ref(&self) -> &str {
        match self {
            SpacingDelimiter::Empty => " ",
            SpacingDelimiter::Comma => ", ",
            SpacingDelimiter::Colon => ": ",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExpandableDelimiter {
    CollapsedArray,
    CollapsedEmptyArray,
    OpeningArray,
    ClosingArray,
    CollapsedObject,
    CollapsedEmptyObject,
    OpeningObject,
    ClosingObject,
}

impl AsRef<str> for ExpandableDelimiter {
    fn as_ref(&self) -> &str {
        match self {
            ExpandableDelimiter::CollapsedArray => "[...]",
            ExpandableDelimiter::CollapsedEmptyArray => "[]",
            ExpandableDelimiter::OpeningArray => "[",
            ExpandableDelimiter::ClosingArray => "]",
            ExpandableDelimiter::CollapsedObject => "{...}",
            ExpandableDelimiter::CollapsedEmptyObject => "{}",
            ExpandableDelimiter::OpeningObject => "{",
            ExpandableDelimiter::ClosingObject => "}",
        }
    }
}
