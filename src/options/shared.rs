//! Shared enums used by both [`super::DecodeOptions`] and
//! [`super::EncodeOptions`].

use regex::Regex;

/// The character set used when encoding or decoding percent-escaped text.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Charset {
    /// UTF-8 semantics.
    #[default]
    Utf8,
    /// ISO-8859-1 semantics.
    Iso88591,
}

impl Charset {
    pub(crate) const UTF8_SENTINEL: &str = "utf8=%E2%9C%93";
    pub(crate) const ISO_SENTINEL: &str = "utf8=%26%2310003%3B";

    pub(crate) fn sentinel(self) -> &'static str {
        match self {
            Self::Utf8 => Self::UTF8_SENTINEL,
            Self::Iso88591 => Self::ISO_SENTINEL,
        }
    }
}

/// The percent-encoding flavor used when building query strings.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Format {
    /// Percent-encode spaces as `%20`.
    #[default]
    Rfc3986,
    /// Percent-encode spaces as `+`.
    Rfc1738,
}

/// The list notation used when encoding arrays.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ListFormat {
    /// Encode arrays as `a[0]=x&a[1]=y`.
    #[default]
    Indices,
    /// Encode arrays as `a[]=x&a[]=y`.
    Brackets,
    /// Encode arrays as `a=x&a=y`.
    Repeat,
    /// Encode arrays as `a=x,y`.
    Comma,
}

/// The strategy used when the same key appears multiple times during decode.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Duplicates {
    /// Combine repeated values into an array when possible.
    #[default]
    Combine,
    /// Keep the first occurrence and ignore later ones.
    First,
    /// Keep the last occurrence and overwrite earlier ones.
    Last,
}

/// The built-in key ordering mode for encoding objects.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SortMode {
    /// Preserve the original insertion order.
    #[default]
    Preserve,
    /// Sort keys lexicographically in ascending order.
    LexicographicAsc,
}

/// Identifies whether a custom decoder is processing a key or a value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeKind {
    /// The current input is a key component.
    Key,
    /// The current input is a value component.
    Value,
}

/// The query-string delimiter used during decode.
#[derive(Clone, Debug)]
pub enum Delimiter {
    /// Split on a literal string delimiter such as `&` or `;`.
    String(String),
    /// Split on regex matches.
    Regex(Regex),
}

impl Default for Delimiter {
    fn default() -> Self {
        Self::String("&".to_owned())
    }
}

/// A whitelist entry used to select object keys or array indices during
/// encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WhitelistSelector {
    /// Select an object key.
    Key(String),
    /// Select an array index.
    Index(usize),
}
