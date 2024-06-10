pub(crate) struct Delimiters {
    pub(crate) collapsed: Punc<'static>,
    pub(crate) collapsed_empty: Punc<'static>,
    pub(crate) opening: Punc<'static>,
    pub(crate) closing: Punc<'static>,
}

pub(crate) const ARRAY_DELIMITERS: Delimiters = Delimiters {
    collapsed: Punc::CollapsedDelimiter("[...]"),
    collapsed_empty: Punc::CollapsedDelimiter("[]"),
    opening: Punc::OpeningDelimiter("["),
    closing: Punc::ClosingDelimiter("]"),
};

pub(crate) const OBJECT_DELIMITERS: Delimiters = Delimiters {
    collapsed: Punc::CollapsedDelimiter("{...}"),
    collapsed_empty: Punc::CollapsedDelimiter("{}"),
    opening: Punc::OpeningDelimiter("{"),
    closing: Punc::ClosingDelimiter("}"),
};

pub(crate) const EMPTY_SPACE: Punc = Punc::Spacing(" ");
pub(crate) const COMMA_SPACE: Punc = Punc::Spacing(", ");

#[derive(Clone, Copy)]
pub(crate) enum Punc<'a> {
    Spacing(&'a str),
    CollapsedDelimiter(&'a str),
    OpeningDelimiter(&'a str),
    ClosingDelimiter(&'a str),
}

impl<'a> AsRef<str> for Punc<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Punc::Spacing(s) => s,
            Punc::CollapsedDelimiter(s) => s,
            Punc::OpeningDelimiter(s) => s,
            Punc::ClosingDelimiter(s) => s,
        }
    }
}
