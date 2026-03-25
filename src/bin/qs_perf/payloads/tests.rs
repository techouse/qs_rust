use super::build_query;

#[test]
fn build_query_includes_utf8_sentinel_and_comma_payloads() {
    let query = build_query(12, true, true, 8);
    assert!(query.starts_with("utf8=%E2%9C%93&"));
    assert!(query.contains("k0=a,b,c"));
    assert!(query.contains("k10=a,b,c"));
}
