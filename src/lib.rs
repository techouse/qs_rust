#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod compact;
mod decode;
mod encode;
mod error;
pub(crate) mod internal;
mod key_path;
mod merge;
mod options;
mod structured_scan;
mod temporal;
mod value;

#[cfg(feature = "chrono")]
pub mod chrono_support;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "time")]
pub mod time_support;

pub use crate::decode::{decode, decode_pairs};
pub use crate::encode::encode;
pub use crate::error::{DecodeError, EncodeError};
pub use crate::options::{
    Charset, DecodeDecoder, DecodeKind, DecodeOptions, Delimiter, Duplicates, EncodeFilter,
    EncodeOptions, EncodeToken, EncodeTokenEncoder, FilterResult, Format, FunctionFilter,
    ListFormat, SortMode, Sorter, TemporalSerializer, WhitelistSelector,
};
#[cfg(feature = "serde")]
pub use crate::serde::{from_str, to_string};
#[cfg(feature = "serde")]
pub use crate::serde::{from_value, to_value};
pub use crate::temporal::{DateTimeValue, TemporalValue, TemporalValueError};
pub use crate::value::{Object, Value};
