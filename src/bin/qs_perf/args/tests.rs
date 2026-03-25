use super::{Args, OutputFormat, Scenario, parse_args_from};

#[test]
fn args_shape_is_stable_for_output_writes() {
    let args = Args {
        scenario: Scenario::Encode,
        format: OutputFormat::Json,
        warmups: 0,
        samples: 1,
        decode_case: None,
        max_encode_depth: Some(2000),
        output: Some("out.json".to_owned()),
    };

    assert_eq!(args.scenario, Scenario::Encode);
    assert_eq!(args.format, OutputFormat::Json);
    assert_eq!(args.decode_case, None);
    assert_eq!(args.max_encode_depth, Some(2000));
    assert_eq!(args.output.as_deref(), Some("out.json"));
}

#[test]
fn decode_case_filter_is_parsed_from_args() {
    let args = parse_args_from([
        "--scenario",
        "decode",
        "--format",
        "json",
        "--decode-case",
        "C2",
    ]);

    assert_eq!(args.scenario, Scenario::Decode);
    assert_eq!(args.format, OutputFormat::Json);
    assert_eq!(args.decode_case.map(|case| case.name), Some("C2"));
}
