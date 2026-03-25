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
