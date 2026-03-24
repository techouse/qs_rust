mod support;

use crate::support::{
    assert_imported_node_encode_case, cases::imported_encode_cases, require_node_comparison,
};

#[test]
fn typed_encode_parity_matches_node_qs() {
    if !require_node_comparison("typed encode parity") {
        return;
    }

    for case in imported_encode_cases() {
        assert_imported_node_encode_case(&case);
    }
}
