pub(crate) struct Delimiters {
    pub(crate) collapsed: ExpandablePunc,
    pub(crate) collapsed_empty: ExpandablePunc,
    pub(crate) opening: ExpandablePunc,
    pub(crate) closing: ExpandablePunc,
}

pub(crate) const ARRAY_DELIMITERS: Delimiters = Delimiters {
    collapsed: ExpandablePunc::CollapsedArray("[...]"),
    collapsed_empty: ExpandablePunc::CollapsedArray("[]"),
    opening: ExpandablePunc::OpeningArray("["),
    closing: ExpandablePunc::ClosingArray("]"),
};

pub(crate) const OBJECT_DELIMITERS: Delimiters = Delimiters {
    collapsed: ExpandablePunc::CollapsedObject("{...}"),
    collapsed_empty: ExpandablePunc::CollapsedObject("{}"),
    opening: ExpandablePunc::OpeningObject("{"),
    closing: ExpandablePunc::ClosingObject("}"),
};

pub(crate) const EMPTY_SPACE: SpacingPunc = SpacingPunc(" ");
pub(crate) const COMMA_SPACE: SpacingPunc = SpacingPunc(", ");
pub(crate) const COLON_SPACE: SpacingPunc = SpacingPunc(": ");

#[derive(Clone, Copy)]
pub(crate) struct SpacingPunc(&'static str);

impl AsRef<str> for SpacingPunc {
    fn as_ref(&self) -> &str {
        self.0
    }
}

#[derive(Clone, Copy)]
pub enum ExpandablePunc {
    CollapsedArray(&'static str),
    OpeningArray(&'static str),
    ClosingArray(&'static str),
    CollapsedObject(&'static str),
    OpeningObject(&'static str),
    ClosingObject(&'static str),
}

impl AsRef<str> for ExpandablePunc {
    fn as_ref(&self) -> &str {
        match self {
            ExpandablePunc::CollapsedArray(s) => s,
            ExpandablePunc::OpeningArray(s) => s,
            ExpandablePunc::ClosingArray(s) => s,
            ExpandablePunc::CollapsedObject(s) => s,
            ExpandablePunc::OpeningObject(s) => s,
            ExpandablePunc::ClosingObject(s) => s,
        }
    }
}
