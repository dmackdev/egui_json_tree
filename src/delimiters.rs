pub(crate) struct Delimiters {
    pub(crate) collapsed: ExpandablePunc,
    pub(crate) collapsed_empty: ExpandablePunc,
    pub(crate) opening: ExpandablePunc,
    pub(crate) closing: ExpandablePunc,
}

pub(crate) const ARRAY_DELIMITERS: Delimiters = Delimiters {
    collapsed: ExpandablePunc::CollapsedArray,
    collapsed_empty: ExpandablePunc::CollapsedEmptyArray,
    opening: ExpandablePunc::OpeningArray,
    closing: ExpandablePunc::ClosingArray,
};

pub(crate) const OBJECT_DELIMITERS: Delimiters = Delimiters {
    collapsed: ExpandablePunc::CollapsedObject,
    collapsed_empty: ExpandablePunc::CollapsedEmptyObject,
    opening: ExpandablePunc::OpeningObject,
    closing: ExpandablePunc::ClosingObject,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum SpacingPunc {
    Empty,
    Comma,
    Colon,
}

impl AsRef<str> for SpacingPunc {
    fn as_ref(&self) -> &str {
        match self {
            SpacingPunc::Empty => " ",
            SpacingPunc::Comma => ", ",
            SpacingPunc::Colon => ": ",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExpandablePunc {
    CollapsedArray,
    CollapsedEmptyArray,
    OpeningArray,
    ClosingArray,
    CollapsedObject,
    CollapsedEmptyObject,
    OpeningObject,
    ClosingObject,
}

impl AsRef<str> for ExpandablePunc {
    fn as_ref(&self) -> &str {
        match self {
            ExpandablePunc::CollapsedArray => "[...]",
            ExpandablePunc::CollapsedEmptyArray => "[]",
            ExpandablePunc::OpeningArray => "[",
            ExpandablePunc::ClosingArray => "]",
            ExpandablePunc::CollapsedObject => "{...}",
            ExpandablePunc::CollapsedEmptyObject => "{}",
            ExpandablePunc::OpeningObject => "{",
            ExpandablePunc::ClosingObject => "}",
        }
    }
}
