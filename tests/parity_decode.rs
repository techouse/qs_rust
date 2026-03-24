mod support;

use crate::support::{
    assert_imported_node_decode_case, cases::imported_decode_cases, require_node_comparison,
};

#[test]
fn typed_decode_parity_matches_node_qs() {
    if !require_node_comparison("typed decode parity") {
        return;
    }

    for case in imported_decode_cases() {
        assert_imported_node_decode_case(&case);
    }
}
