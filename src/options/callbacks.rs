//! Callback wrappers used by the public option types.

use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

use crate::temporal::TemporalValue;
use crate::value::Value;

use super::shared::{Charset, DecodeKind, Format, WhitelistSelector};

type DecodeDecoderFn = dyn Fn(&str, Charset, DecodeKind) -> String + Send + Sync;
type EncodeTokenEncoderFn = dyn Fn(EncodeToken<'_>, Charset, Format) -> String + Send + Sync;
type FunctionFilterFn = dyn Fn(&str, &Value) -> FilterResult + Send + Sync;
type SorterFn = dyn Fn(&str, &str) -> Ordering + Send + Sync;
type TemporalSerializerFn = dyn Fn(&TemporalValue) -> Option<String> + Send + Sync;

/// A custom decode callback for transforming raw key and value components.
///
/// The callback receives the raw percent-encoded input, the selected
/// [`Charset`], and whether the input is a [`DecodeKind::Key`] or
/// [`DecodeKind::Value`].
#[derive(Clone)]
pub struct DecodeDecoder(Arc<DecodeDecoderFn>);

impl DecodeDecoder {
    /// Wraps a custom decode callback.
    ///
    /// # Examples
    ///
    /// ```
    /// use qs_rust::{Charset, DecodeDecoder, DecodeKind};
    ///
    /// let decoder = DecodeDecoder::new(|input, _charset, kind| match kind {
    ///     DecodeKind::Key => input.to_ascii_uppercase(),
    ///     DecodeKind::Value => input.to_owned(),
    /// });
    ///
    /// assert_eq!(decoder.decode("a", Charset::Utf8, DecodeKind::Key), "A");
    /// ```
    pub fn new<F>(decoder: F) -> Self
    where
        F: Fn(&str, Charset, DecodeKind) -> String + Send + Sync + 'static,
    {
        Self(Arc::new(decoder))
    }

    /// Invokes the wrapped callback.
    pub fn decode(&self, input: &str, charset: Charset, kind: DecodeKind) -> String {
        (self.0)(input, charset, kind)
    }
}

impl fmt::Debug for DecodeDecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DecodeDecoder(<closure>)")
    }
}

/// A key or value token presented to [`EncodeTokenEncoder`].
#[derive(Clone, Copy, Debug)]
pub enum EncodeToken<'a> {
    /// A materialized key path before final formatter percent-encoding.
    Key(&'a str),
    /// A normal value from the dynamic [`Value`] tree.
    Value(&'a Value),
    /// An already-stringified value token, used for joined comma-list values.
    TextValue(&'a str),
}

/// A custom key/value encoder used by [`super::EncodeOptions`].
#[derive(Clone)]
pub struct EncodeTokenEncoder(Arc<EncodeTokenEncoderFn>);

impl EncodeTokenEncoder {
    /// Wraps a custom key/value encoding callback.
    pub fn new<F>(encoder: F) -> Self
    where
        F: Fn(EncodeToken<'_>, Charset, Format) -> String + Send + Sync + 'static,
    {
        Self(Arc::new(encoder))
    }

    /// Invokes the wrapped callback.
    pub fn encode(&self, token: EncodeToken<'_>, charset: Charset, format: Format) -> String {
        (self.0)(token, charset, format)
    }
}

impl fmt::Debug for EncodeTokenEncoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("EncodeTokenEncoder(<closure>)")
    }
}

/// The outcome of a function-based encode filter.
#[derive(Clone, Debug, PartialEq)]
pub enum FilterResult {
    /// Keep the value unchanged.
    Keep,
    /// Omit the value entirely.
    Omit,
    /// Replace the value before encoding continues.
    Replace(Value),
}

/// A callback used to filter or replace values during encoding.
#[derive(Clone)]
pub struct FunctionFilter(Arc<FunctionFilterFn>);

impl FunctionFilter {
    /// Wraps a function filter callback.
    pub fn new<F>(filter: F) -> Self
    where
        F: Fn(&str, &Value) -> FilterResult + Send + Sync + 'static,
    {
        Self(Arc::new(filter))
    }

    /// Invokes the wrapped filter callback.
    pub fn apply(&self, prefix: &str, value: &Value) -> FilterResult {
        (self.0)(prefix, value)
    }
}

impl fmt::Debug for FunctionFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FunctionFilter(<closure>)")
    }
}

/// The public filtering modes supported by the encoder.
#[derive(Clone, Debug)]
pub enum EncodeFilter {
    /// Only encode the listed object keys or array indices.
    Whitelist(Vec<WhitelistSelector>),
    /// Decide per value whether to keep, omit, or replace it.
    Function(FunctionFilter),
}

/// A callback used to compare two object keys during encoding.
#[derive(Clone)]
pub struct Sorter(Arc<SorterFn>);

impl Sorter {
    /// Wraps a custom key sorter.
    pub fn new<F>(sorter: F) -> Self
    where
        F: Fn(&str, &str) -> Ordering + Send + Sync + 'static,
    {
        Self(Arc::new(sorter))
    }

    /// Invokes the wrapped comparator.
    pub fn compare(&self, left: &str, right: &str) -> Ordering {
        (self.0)(left, right)
    }
}

impl fmt::Debug for Sorter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Sorter(<closure>)")
    }
}

/// A callback used to customize temporal serialization before encoding.
#[derive(Clone)]
pub struct TemporalSerializer(Arc<TemporalSerializerFn>);

impl TemporalSerializer {
    /// Wraps a custom temporal serializer.
    pub fn new<F>(serializer: F) -> Self
    where
        F: Fn(&TemporalValue) -> Option<String> + Send + Sync + 'static,
    {
        Self(Arc::new(serializer))
    }

    /// Invokes the wrapped serializer.
    pub fn serialize(&self, value: &TemporalValue) -> Option<String> {
        (self.0)(value)
    }
}

impl fmt::Debug for TemporalSerializer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("TemporalSerializer(<closure>)")
    }
}
