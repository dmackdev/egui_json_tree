//! Tokens for array brackets and object braces used during rendering.

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
            Self::Empty => " ",
            Self::Comma => ", ",
            Self::Colon => ": ",
        }
    }
}

/// Tokens for array brackets and object braces.
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
            Self::CollapsedArray => "[...]",
            Self::CollapsedEmptyArray => "[]",
            Self::OpeningArray => "[",
            Self::ClosingArray => "]",
            Self::CollapsedObject => "{...}",
            Self::CollapsedEmptyObject => "{}",
            Self::OpeningObject => "{",
            Self::ClosingObject => "}",
        }
    }
}
