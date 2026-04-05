use super::{first_structured_split_index, leading_structured_root, scan_structured_keys};
use crate::options::DecodeOptions;

#[test]
fn structured_scan_helpers_match_expected_behavior() {
    assert_eq!(first_structured_split_index("a%2eb%2Ec", true), Some(1));
    assert_eq!(
        leading_structured_root("a[b]=1", &DecodeOptions::default()).unwrap(),
        "a"
    );
    assert_eq!(
        leading_structured_root("[]=x", &DecodeOptions::default()).unwrap(),
        "0"
    );

    let scan = scan_structured_keys(["a[b]", "plain"], &DecodeOptions::default()).unwrap();
    assert!(scan.has_any_structured_syntax);
    assert!(scan.contains_structured_key("a[b]"));
    assert!(scan.contains_structured_root("a"));
    assert!(!scan.contains_structured_root("plain"));
}

#[test]
fn structured_scan_treats_encoded_dots_as_structured_roots() {
    let options = DecodeOptions::new().with_allow_dots(true);
    let scan = scan_structured_keys(["a%2Eb", "plain"], &options).unwrap();

    assert!(scan.has_any_structured_syntax);
    assert!(scan.contains_structured_key("a%2Eb"));
    assert!(scan.contains_structured_root("a"));
    assert!(!scan.contains_structured_root("plain"));
}

#[test]
fn leading_structured_root_preserves_noncanonical_numeric_keys() {
    assert_eq!(
        leading_structured_root("[01]=x", &DecodeOptions::default()).unwrap(),
        "01"
    );
}

#[test]
fn structured_scan_helpers_cover_plain_percent_inputs_and_empty_segments() {
    assert_eq!(first_structured_split_index("plain%20text", true), None);
    assert_eq!(
        leading_structured_root("", &DecodeOptions::default()).unwrap(),
        ""
    );
}
