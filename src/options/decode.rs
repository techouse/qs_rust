//! Decode-specific configuration.

use crate::error::DecodeError;

use super::callbacks::DecodeDecoder;
use super::shared::{Charset, Delimiter, Duplicates};

/// Options that control query-string decoding.
///
/// The defaults are chosen to match the common `qs` behavior: UTF-8 decoding,
/// `&` as the delimiter, five levels of depth, list parsing enabled, and
/// duplicate values combined into arrays.
#[derive(Clone, Debug)]
pub struct DecodeOptions {
    pub(crate) allow_dots: bool,
    pub(crate) decode_dot_in_keys: bool,
    pub(crate) allow_empty_lists: bool,
    pub(crate) allow_sparse_lists: bool,
    pub(crate) list_limit: usize,
    pub(crate) charset: Charset,
    pub(crate) charset_sentinel: bool,
    pub(crate) comma: bool,
    pub(crate) delimiter: Delimiter,
    pub(crate) depth: usize,
    pub(crate) parameter_limit: usize,
    pub(crate) duplicates: Duplicates,
    pub(crate) ignore_query_prefix: bool,
    pub(crate) interpret_numeric_entities: bool,
    pub(crate) parse_lists: bool,
    pub(crate) strict_depth: bool,
    pub(crate) strict_null_handling: bool,
    pub(crate) throw_on_limit_exceeded: bool,
    pub(crate) decoder: Option<DecodeDecoder>,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            allow_dots: false,
            decode_dot_in_keys: false,
            allow_empty_lists: false,
            allow_sparse_lists: false,
            list_limit: 20,
            charset: Charset::Utf8,
            charset_sentinel: false,
            comma: false,
            delimiter: Delimiter::default(),
            depth: 5,
            parameter_limit: 1000,
            duplicates: Duplicates::Combine,
            ignore_query_prefix: false,
            interpret_numeric_entities: false,
            parse_lists: true,
            strict_depth: false,
            strict_null_handling: false,
            throw_on_limit_exceeded: false,
            decoder: None,
        }
    }
}

impl DecodeOptions {
    /// Creates a new option set with the default decode configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns whether dots in keys are treated as path separators.
    pub fn allow_dots(&self) -> bool {
        self.allow_dots
    }

    /// Enables or disables dot notation during decode.
    ///
    /// Setting this to `false` also clears
    /// [`Self::decode_dot_in_keys`].
    pub fn with_allow_dots(mut self, allow_dots: bool) -> Self {
        self.allow_dots = allow_dots;
        if !allow_dots {
            self.decode_dot_in_keys = false;
        }
        self
    }

    /// Returns whether literal dots inside keys are decoded from `%2E` when
    /// dot notation is enabled.
    pub fn decode_dot_in_keys(&self) -> bool {
        self.decode_dot_in_keys
    }

    /// Enables or disables percent-decoding of dots inside key segments.
    ///
    /// Enabling this option also enables [`Self::allow_dots`].
    pub fn with_decode_dot_in_keys(mut self, decode_dot_in_keys: bool) -> Self {
        self.decode_dot_in_keys = decode_dot_in_keys;
        if decode_dot_in_keys {
            self.allow_dots = true;
        }
        self
    }

    /// Returns whether empty list assignments such as `a[]` are preserved.
    pub fn allow_empty_lists(&self) -> bool {
        self.allow_empty_lists
    }

    /// Enables or disables preservation of empty list assignments.
    pub fn with_allow_empty_lists(mut self, allow_empty_lists: bool) -> Self {
        self.allow_empty_lists = allow_empty_lists;
        self
    }

    /// Returns whether sparse list indices are preserved instead of compacted.
    pub fn allow_sparse_lists(&self) -> bool {
        self.allow_sparse_lists
    }

    /// Enables or disables sparse list preservation.
    pub fn with_allow_sparse_lists(mut self, allow_sparse_lists: bool) -> Self {
        self.allow_sparse_lists = allow_sparse_lists;
        self
    }

    /// Returns the maximum dense list length accepted before overflow handling
    /// is required.
    pub fn list_limit(&self) -> usize {
        self.list_limit
    }

    /// Sets the maximum dense list length accepted during decode.
    pub fn with_list_limit(mut self, list_limit: usize) -> Self {
        self.list_limit = list_limit;
        self
    }

    /// Returns the default character set used for percent-decoding.
    pub fn charset(&self) -> Charset {
        self.charset
    }

    /// Sets the default character set used for percent-decoding.
    pub fn with_charset(mut self, charset: Charset) -> Self {
        self.charset = charset;
        self
    }

    /// Returns whether charset sentinel pairs are honored.
    pub fn charset_sentinel(&self) -> bool {
        self.charset_sentinel
    }

    /// Enables or disables charset sentinel handling.
    pub fn with_charset_sentinel(mut self, charset_sentinel: bool) -> Self {
        self.charset_sentinel = charset_sentinel;
        self
    }

    /// Returns whether comma-separated values are treated as lists.
    pub fn comma(&self) -> bool {
        self.comma
    }

    /// Enables or disables comma-list parsing.
    pub fn with_comma(mut self, comma: bool) -> Self {
        self.comma = comma;
        self
    }

    /// Returns the configured query-string delimiter.
    pub fn delimiter(&self) -> &Delimiter {
        &self.delimiter
    }

    /// Sets the query-string delimiter used during raw scanning.
    pub fn with_delimiter(mut self, delimiter: Delimiter) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Returns the configured maximum structured depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Sets the maximum structured depth considered during decode.
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Returns the maximum number of parameters accepted from the raw input.
    pub fn parameter_limit(&self) -> usize {
        self.parameter_limit
    }

    /// Sets the maximum number of parameters accepted from the raw input.
    pub fn with_parameter_limit(mut self, parameter_limit: usize) -> Self {
        self.parameter_limit = parameter_limit;
        self
    }

    /// Returns the duplicate-handling strategy.
    pub fn duplicates(&self) -> Duplicates {
        self.duplicates
    }

    /// Sets the duplicate-handling strategy.
    pub fn with_duplicates(mut self, duplicates: Duplicates) -> Self {
        self.duplicates = duplicates;
        self
    }

    /// Returns whether a leading `?` is ignored.
    pub fn ignore_query_prefix(&self) -> bool {
        self.ignore_query_prefix
    }

    /// Enables or disables ignoring a leading `?` in the input.
    pub fn with_ignore_query_prefix(mut self, ignore_query_prefix: bool) -> Self {
        self.ignore_query_prefix = ignore_query_prefix;
        self
    }

    /// Returns whether HTML numeric entities are interpreted in ISO-8859-1
    /// mode.
    pub fn interpret_numeric_entities(&self) -> bool {
        self.interpret_numeric_entities
    }

    /// Enables or disables interpretation of HTML numeric entities.
    pub fn with_interpret_numeric_entities(mut self, interpret_numeric_entities: bool) -> Self {
        self.interpret_numeric_entities = interpret_numeric_entities;
        self
    }

    /// Returns whether bracketed numeric segments are parsed as lists.
    pub fn parse_lists(&self) -> bool {
        self.parse_lists
    }

    /// Enables or disables list parsing from bracketed numeric segments.
    pub fn with_parse_lists(mut self, parse_lists: bool) -> Self {
        self.parse_lists = parse_lists;
        self
    }

    /// Returns whether exceeding the configured depth is treated as an error.
    pub fn strict_depth(&self) -> bool {
        self.strict_depth
    }

    /// Enables or disables strict depth enforcement.
    pub fn with_strict_depth(mut self, strict_depth: bool) -> Self {
        self.strict_depth = strict_depth;
        self
    }

    /// Returns whether missing values are preserved as [`crate::Value::Null`].
    pub fn strict_null_handling(&self) -> bool {
        self.strict_null_handling
    }

    /// Enables or disables strict null handling for missing values.
    pub fn with_strict_null_handling(mut self, strict_null_handling: bool) -> Self {
        self.strict_null_handling = strict_null_handling;
        self
    }

    /// Returns whether limit overflows are reported as errors instead of being
    /// compacted or redirected.
    pub fn throw_on_limit_exceeded(&self) -> bool {
        self.throw_on_limit_exceeded
    }

    /// Enables or disables hard errors for list and parameter limit overflows.
    pub fn with_throw_on_limit_exceeded(mut self, throw_on_limit_exceeded: bool) -> Self {
        self.throw_on_limit_exceeded = throw_on_limit_exceeded;
        self
    }

    /// Returns the custom decoder callback, if one is configured.
    pub fn decoder(&self) -> Option<&DecodeDecoder> {
        self.decoder.as_ref()
    }

    /// Sets an optional custom decoder callback.
    pub fn with_decoder(mut self, decoder: Option<DecodeDecoder>) -> Self {
        self.decoder = decoder;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), DecodeError> {
        if self.parameter_limit == 0 {
            return Err(DecodeError::InvalidParameterLimit);
        }

        if self.decode_dot_in_keys && !self.allow_dots {
            return Err(DecodeError::DecodeDotInKeysRequiresAllowDots);
        }

        if matches!(&self.delimiter, Delimiter::String(text) if text.is_empty()) {
            return Err(DecodeError::EmptyDelimiter);
        }

        Ok(())
    }
}
