//! Encode-specific configuration.

use crate::error::EncodeError;

use super::callbacks::{EncodeFilter, EncodeTokenEncoder, Sorter, TemporalSerializer};
use super::shared::{Charset, Format, ListFormat, SortMode, WhitelistSelector};

/// Options that control query-string encoding.
///
/// The defaults follow the common `qs` behavior: RFC 3986 percent-encoding,
/// `&` delimiters, indexed list notation, and insertion-order output.
#[derive(Clone, Debug)]
pub struct EncodeOptions {
    pub(crate) encode: bool,
    pub(crate) delimiter: String,
    pub(crate) list_format: ListFormat,
    pub(crate) format: Format,
    pub(crate) charset: Charset,
    pub(crate) charset_sentinel: bool,
    pub(crate) allow_empty_lists: bool,
    pub(crate) strict_null_handling: bool,
    pub(crate) skip_nulls: bool,
    pub(crate) comma_round_trip: bool,
    pub(crate) comma_compact_nulls: bool,
    pub(crate) encode_values_only: bool,
    pub(crate) add_query_prefix: bool,
    pub(crate) allow_dots: bool,
    pub(crate) encode_dot_in_keys: bool,
    pub(crate) filter: Option<EncodeFilter>,
    pub(crate) sort: SortMode,
    pub(crate) sorter: Option<Sorter>,
    pub(crate) encoder: Option<EncodeTokenEncoder>,
    pub(crate) temporal_serializer: Option<TemporalSerializer>,
    pub(crate) max_depth: Option<usize>,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            encode: true,
            delimiter: "&".to_owned(),
            list_format: ListFormat::Indices,
            format: Format::Rfc3986,
            charset: Charset::Utf8,
            charset_sentinel: false,
            allow_empty_lists: false,
            strict_null_handling: false,
            skip_nulls: false,
            comma_round_trip: false,
            comma_compact_nulls: false,
            encode_values_only: false,
            add_query_prefix: false,
            allow_dots: false,
            encode_dot_in_keys: false,
            filter: None,
            sort: SortMode::Preserve,
            sorter: None,
            encoder: None,
            temporal_serializer: None,
            max_depth: None,
        }
    }
}

impl EncodeOptions {
    /// Creates a new option set with the default encode configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether scalar values are percent-encoded.
    pub fn encode(&self) -> bool {
        self.encode
    }

    /// Enables or disables percent-encoding.
    pub fn with_encode(mut self, encode: bool) -> Self {
        self.encode = encode;
        self
    }

    /// Returns the delimiter inserted between encoded pairs.
    pub fn delimiter(&self) -> &str {
        &self.delimiter
    }

    /// Sets the delimiter inserted between encoded pairs.
    pub fn with_delimiter<S>(mut self, delimiter: S) -> Self
    where
        S: Into<String>,
    {
        self.delimiter = delimiter.into();
        self
    }

    /// Returns the list notation used for arrays.
    pub fn list_format(&self) -> ListFormat {
        self.list_format
    }

    /// Sets the list notation used for arrays.
    pub fn with_list_format(mut self, list_format: ListFormat) -> Self {
        self.list_format = list_format;
        self
    }

    /// Returns the percent-encoding flavor used for strings.
    pub fn format(&self) -> Format {
        self.format
    }

    /// Sets the percent-encoding flavor used for strings.
    pub fn with_format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Returns the character set used for percent-encoding.
    pub fn charset(&self) -> Charset {
        self.charset
    }

    /// Sets the character set used for percent-encoding.
    pub fn with_charset(mut self, charset: Charset) -> Self {
        self.charset = charset;
        self
    }

    /// Returns whether a charset sentinel is prefixed to the output.
    pub fn charset_sentinel(&self) -> bool {
        self.charset_sentinel
    }

    /// Enables or disables prepending a charset sentinel pair.
    pub fn with_charset_sentinel(mut self, charset_sentinel: bool) -> Self {
        self.charset_sentinel = charset_sentinel;
        self
    }

    /// Returns whether empty arrays are emitted.
    pub fn allow_empty_lists(&self) -> bool {
        self.allow_empty_lists
    }

    /// Enables or disables emitting empty arrays.
    pub fn with_allow_empty_lists(mut self, allow_empty_lists: bool) -> Self {
        self.allow_empty_lists = allow_empty_lists;
        self
    }

    /// Returns whether nulls are emitted without `=`.
    pub fn strict_null_handling(&self) -> bool {
        self.strict_null_handling
    }

    /// Enables or disables strict null handling.
    pub fn with_strict_null_handling(mut self, strict_null_handling: bool) -> Self {
        self.strict_null_handling = strict_null_handling;
        self
    }

    /// Returns whether null values are omitted entirely.
    pub fn skip_nulls(&self) -> bool {
        self.skip_nulls
    }

    /// Enables or disables omission of null values.
    pub fn with_skip_nulls(mut self, skip_nulls: bool) -> Self {
        self.skip_nulls = skip_nulls;
        self
    }

    /// Returns whether single-item comma lists are round-tripped with `[]`.
    pub fn comma_round_trip(&self) -> bool {
        self.comma_round_trip
    }

    /// Enables or disables comma round-tripping for single-item arrays.
    pub fn with_comma_round_trip(mut self, comma_round_trip: bool) -> Self {
        self.comma_round_trip = comma_round_trip;
        self
    }

    /// Returns whether nulls are compacted out of comma-encoded arrays.
    pub fn comma_compact_nulls(&self) -> bool {
        self.comma_compact_nulls
    }

    /// Enables or disables null compaction for comma-encoded arrays.
    pub fn with_comma_compact_nulls(mut self, comma_compact_nulls: bool) -> Self {
        self.comma_compact_nulls = comma_compact_nulls;
        self
    }

    /// Returns whether only values, and not keys, are percent-encoded.
    pub fn encode_values_only(&self) -> bool {
        self.encode_values_only
    }

    /// Enables or disables value-only percent-encoding.
    pub fn with_encode_values_only(mut self, encode_values_only: bool) -> Self {
        self.encode_values_only = encode_values_only;
        self
    }

    /// Returns whether a leading `?` is prefixed to the output.
    pub fn add_query_prefix(&self) -> bool {
        self.add_query_prefix
    }

    /// Enables or disables a leading `?` in the encoded output.
    pub fn with_add_query_prefix(mut self, add_query_prefix: bool) -> Self {
        self.add_query_prefix = add_query_prefix;
        self
    }

    /// Returns whether nested object paths are encoded with dot notation.
    pub fn allow_dots(&self) -> bool {
        self.allow_dots
    }

    /// Enables or disables dot notation during encode.
    ///
    /// Setting this to `false` also clears [`Self::encode_dot_in_keys`].
    pub fn with_allow_dots(mut self, allow_dots: bool) -> Self {
        self.allow_dots = allow_dots;
        if !allow_dots {
            self.encode_dot_in_keys = false;
        }
        self
    }

    /// Returns whether literal dots in key names are percent-encoded when dot
    /// notation is active.
    pub fn encode_dot_in_keys(&self) -> bool {
        self.encode_dot_in_keys
    }

    /// Enables or disables percent-encoding of literal dots in keys.
    ///
    /// Enabling this option also enables [`Self::allow_dots`].
    pub fn with_encode_dot_in_keys(mut self, encode_dot_in_keys: bool) -> Self {
        self.encode_dot_in_keys = encode_dot_in_keys;
        if encode_dot_in_keys {
            self.allow_dots = true;
        }
        self
    }

    /// Returns the configured filter, if any.
    pub fn filter(&self) -> Option<&EncodeFilter> {
        self.filter.as_ref()
    }

    /// Sets an optional encode filter.
    pub fn with_filter(mut self, filter: Option<EncodeFilter>) -> Self {
        self.filter = filter;
        self
    }

    /// Returns the current whitelist when [`EncodeFilter::Whitelist`] is in
    /// use.
    pub fn whitelist(&self) -> Option<&[WhitelistSelector]> {
        match self.filter.as_ref() {
            Some(EncodeFilter::Whitelist(entries)) => Some(entries),
            _ => None,
        }
    }

    /// Replaces the current filter with a whitelist, or clears it when `None`
    /// is supplied.
    pub fn with_whitelist(mut self, whitelist: Option<Vec<WhitelistSelector>>) -> Self {
        self.filter = whitelist.map(EncodeFilter::Whitelist);
        self
    }

    /// Returns the built-in sort mode.
    pub fn sort(&self) -> SortMode {
        self.sort
    }

    /// Sets the built-in sort mode.
    pub fn with_sort(mut self, sort: SortMode) -> Self {
        self.sort = sort;
        self
    }

    /// Returns the custom sorter, if one is configured.
    pub fn sorter(&self) -> Option<&Sorter> {
        self.sorter.as_ref()
    }

    /// Sets an optional custom sorter.
    pub fn with_sorter(mut self, sorter: Option<Sorter>) -> Self {
        self.sorter = sorter;
        self
    }

    /// Returns the custom key/value encoder, if one is configured.
    pub fn encoder(&self) -> Option<&EncodeTokenEncoder> {
        self.encoder.as_ref()
    }

    /// Sets an optional custom key/value encoder.
    pub fn with_encoder(mut self, encoder: Option<EncodeTokenEncoder>) -> Self {
        self.encoder = encoder;
        self
    }

    /// Returns the custom temporal serializer, if one is configured.
    pub fn temporal_serializer(&self) -> Option<&TemporalSerializer> {
        self.temporal_serializer.as_ref()
    }

    /// Sets an optional custom temporal serializer.
    pub fn with_temporal_serializer(
        mut self,
        temporal_serializer: Option<TemporalSerializer>,
    ) -> Self {
        self.temporal_serializer = temporal_serializer;
        self
    }

    /// Returns the maximum traversal depth, if one is configured.
    pub fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    /// Sets the maximum traversal depth.
    pub fn with_max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), EncodeError> {
        if self.delimiter.is_empty() {
            return Err(EncodeError::EmptyDelimiter);
        }

        if self.encode_dot_in_keys && !self.allow_dots {
            return Err(EncodeError::EncodeDotInKeysRequiresAllowDots);
        }

        Ok(())
    }

    pub(crate) fn has_temporal_serializer(&self) -> bool {
        self.temporal_serializer.is_some()
    }
}

#[cfg(test)]
mod tests;
