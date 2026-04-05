use super::{
    Charset, DecodeOptions, DefaultAccumulator, Duplicates, ScannedPart, Value, finalize_flat,
    scan_default_parts_by_byte_delimiter, scan_string_parts,
};

#[test]
fn string_part_scanners_skip_empty_segments_for_byte_and_multi_byte_delimiters() {
    let mut byte_parts = Vec::new();
    scan_string_parts("a=1&&b=2&", "&", |part: ScannedPart<'_>| {
        byte_parts.push(part.raw_parts().0.to_owned());
        Ok(())
    })
    .unwrap();
    assert_eq!(byte_parts, vec!["a".to_owned(), "b".to_owned()]);

    let mut multi_parts = Vec::new();
    scan_string_parts("a=1&&b=2&&", "&&", |part: ScannedPart<'_>| {
        multi_parts.push(part.raw_parts().0.to_owned());
        Ok(())
    })
    .unwrap();
    assert_eq!(multi_parts, vec!["a".to_owned(), "b".to_owned()]);
}

#[test]
fn default_byte_scanner_routes_plain_and_featureful_parts() {
    let options = DecodeOptions::new()
        .with_charset(Charset::Iso88591)
        .with_interpret_numeric_entities(true)
        .with_duplicates(Duplicates::Last);
    let mut values = DefaultAccumulator::direct();
    let mut token_count = 0usize;
    let mut has_any_structured_syntax = false;

    scan_default_parts_by_byte_delimiter(
        "plain=1|latin=&#65;|encoded%20key=value",
        b'|',
        Charset::Iso88591,
        &options,
        &mut values,
        &mut token_count,
        &mut has_any_structured_syntax,
    )
    .unwrap();

    let decoded = finalize_flat(values.into_flat_values(), &options).unwrap();
    assert_eq!(decoded.get("plain"), Some(&Value::String("1".to_owned())));
    assert_eq!(decoded.get("latin"), Some(&Value::String("A".to_owned())));
    assert_eq!(
        decoded.get("encoded key"),
        Some(&Value::String("value".to_owned()))
    );
}
