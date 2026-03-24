use thiserror::Error;

/// Errors that can occur while decoding a query string into an [`crate::Object`].
///
/// This enum is marked `non_exhaustive` so additional variants can be added in
/// future releases without breaking downstream matches.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DecodeError {
    /// The configured delimiter string was empty.
    #[error("delimiter must not be empty")]
    EmptyDelimiter,

    /// The configured parameter limit was zero.
    #[error("parameter limit must be positive")]
    InvalidParameterLimit,

    /// [`crate::DecodeOptions::decode_dot_in_keys`] was enabled while
    /// [`crate::DecodeOptions::allow_dots`] was disabled.
    #[error("decode_dot_in_keys requires allow_dots to be true")]
    DecodeDotInKeysRequiresAllowDots,

    /// The decoded input exceeded the configured parameter limit.
    #[error("parameter limit exceeded; only {limit} parameter(s) allowed")]
    ParameterLimitExceeded {
        /// The configured maximum number of parameters.
        limit: usize,
    },

    /// A decoded list exceeded the configured list limit.
    #[error("list limit exceeded; only {limit} element(s) allowed in a list")]
    ListLimitExceeded {
        /// The configured maximum list length.
        limit: usize,
    },

    /// The decoded nesting depth exceeded the configured limit.
    #[error("input depth exceeded depth option of {depth} and strict_depth is true")]
    DepthExceeded {
        /// The configured maximum decode depth.
        depth: usize,
    },

    #[cfg(feature = "serde")]
    /// Serde deserialization failed after the query string was decoded into an
    /// intermediate [`crate::Value`] tree.
    #[error("serde decode failed: {0}")]
    Serde(#[from] serde_json::Error),
}

impl DecodeError {
    /// Returns `true` when the error reports that the parameter limit was
    /// exceeded.
    pub fn is_parameter_limit_exceeded(&self) -> bool {
        matches!(self, Self::ParameterLimitExceeded { .. })
    }

    /// Returns the configured parameter limit for
    /// [`Self::ParameterLimitExceeded`].
    pub fn parameter_limit(&self) -> Option<usize> {
        match self {
            Self::ParameterLimitExceeded { limit } => Some(*limit),
            _ => None,
        }
    }

    /// Returns `true` when the error reports that a list limit was exceeded.
    pub fn is_list_limit_exceeded(&self) -> bool {
        matches!(self, Self::ListLimitExceeded { .. })
    }

    /// Returns the configured list limit for [`Self::ListLimitExceeded`].
    pub fn list_limit(&self) -> Option<usize> {
        match self {
            Self::ListLimitExceeded { limit } => Some(*limit),
            _ => None,
        }
    }

    /// Returns `true` when the error reports that the nesting depth limit was
    /// exceeded.
    pub fn is_depth_exceeded(&self) -> bool {
        matches!(self, Self::DepthExceeded { .. })
    }

    /// Returns the configured depth limit for [`Self::DepthExceeded`].
    pub fn depth_limit(&self) -> Option<usize> {
        match self {
            Self::DepthExceeded { depth } => Some(*depth),
            _ => None,
        }
    }
}

/// Errors that can occur while encoding a [`crate::Value`] tree into a query
/// string.
///
/// This enum is marked `non_exhaustive` so additional variants can be added in
/// future releases without breaking downstream matches.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum EncodeError {
    /// The configured delimiter string was empty.
    #[error("delimiter must not be empty")]
    EmptyDelimiter,

    /// [`crate::EncodeOptions::encode_dot_in_keys`] was enabled while
    /// [`crate::EncodeOptions::allow_dots`] was disabled.
    #[error("encode_dot_in_keys requires allow_dots to be true")]
    EncodeDotInKeysRequiresAllowDots,

    /// The encoder exceeded the configured maximum traversal depth.
    #[error("encode depth exceeded max_depth option of {depth}")]
    DepthExceeded {
        /// The configured maximum encode depth.
        depth: usize,
    },

    #[cfg(feature = "serde")]
    /// Serde serialization failed before the value could be encoded into a
    /// query string.
    #[error("serde encode failed: {0}")]
    Serde(#[from] serde_json::Error),
}

impl EncodeError {
    /// Returns `true` when the delimiter configuration was invalid.
    pub fn is_empty_delimiter(&self) -> bool {
        matches!(self, Self::EmptyDelimiter)
    }

    /// Returns `true` when dot-encoding was requested without enabling dot
    /// notation.
    pub fn is_encode_dot_in_keys_requires_allow_dots(&self) -> bool {
        matches!(self, Self::EncodeDotInKeysRequiresAllowDots)
    }

    /// Returns `true` when the encoder exceeded the configured maximum depth.
    pub fn is_depth_exceeded(&self) -> bool {
        matches!(self, Self::DepthExceeded { .. })
    }

    /// Returns the configured depth limit for [`Self::DepthExceeded`].
    pub fn depth_limit(&self) -> Option<usize> {
        match self {
            Self::DepthExceeded { depth } => Some(*depth),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, EncodeError};

    #[test]
    fn decode_error_helper_methods_report_only_the_matching_limits() {
        let parameter = DecodeError::ParameterLimitExceeded { limit: 10 };
        assert!(parameter.is_parameter_limit_exceeded());
        assert_eq!(parameter.parameter_limit(), Some(10));
        assert!(!parameter.is_list_limit_exceeded());
        assert_eq!(parameter.list_limit(), None);
        assert!(!parameter.is_depth_exceeded());
        assert_eq!(parameter.depth_limit(), None);

        let list = DecodeError::ListLimitExceeded { limit: 3 };
        assert!(list.is_list_limit_exceeded());
        assert_eq!(list.list_limit(), Some(3));
        assert!(!list.is_parameter_limit_exceeded());
        assert_eq!(list.parameter_limit(), None);

        let depth = DecodeError::DepthExceeded { depth: 5 };
        assert!(depth.is_depth_exceeded());
        assert_eq!(depth.depth_limit(), Some(5));
        assert!(!depth.is_parameter_limit_exceeded());
        assert_eq!(depth.parameter_limit(), None);
    }

    #[test]
    fn encode_error_helper_methods_report_only_the_matching_limits() {
        let delimiter = EncodeError::EmptyDelimiter;
        assert!(delimiter.is_empty_delimiter());
        assert!(!delimiter.is_encode_dot_in_keys_requires_allow_dots());
        assert!(!delimiter.is_depth_exceeded());
        assert_eq!(delimiter.depth_limit(), None);

        let dots = EncodeError::EncodeDotInKeysRequiresAllowDots;
        assert!(dots.is_encode_dot_in_keys_requires_allow_dots());
        assert!(!dots.is_empty_delimiter());
        assert!(!dots.is_depth_exceeded());

        let depth = EncodeError::DepthExceeded { depth: 7 };
        assert!(depth.is_depth_exceeded());
        assert_eq!(depth.depth_limit(), Some(7));
        assert!(!depth.is_empty_delimiter());
        assert!(!depth.is_encode_dot_in_keys_requires_allow_dots());
    }
}
