use super::{render_snapshot_json, render_snapshot_text};
use crate::cases::{DecodeCase, EncodeCase};
use crate::measure::{DecodeMeasurement, EncodeMeasurement, SampleResult, Snapshot};

#[test]
fn text_renderer_formats_encode_and_decode_sections() {
    let snapshot = Snapshot {
        samples: 3,
        encode: vec![EncodeMeasurement {
            case: EncodeCase {
                depth: 2000,
                iterations: 20,
            },
            result: SampleResult {
                ms_per_op: 1.25,
                output_metric: 42,
            },
        }],
        decode: vec![DecodeMeasurement {
            case: DecodeCase {
                name: "C1",
                count: 100,
                comma: false,
                utf8_sentinel: false,
                value_len: 8,
                iterations: 120,
            },
            result: SampleResult {
                ms_per_op: 0.5,
                output_metric: 100,
            },
        }],
    };

    let rendered = render_snapshot_text(&snapshot);
    assert!(rendered.contains("median of 3 samples"));
    assert!(rendered.contains("depth= 2000:"));
    assert!(rendered.contains("C1: count= 100"));
}

#[test]
fn json_renderer_emits_expected_schema() {
    let snapshot = Snapshot {
        samples: 1,
        encode: vec![EncodeMeasurement {
            case: EncodeCase {
                depth: 2000,
                iterations: 20,
            },
            result: SampleResult {
                ms_per_op: 1.25,
                output_metric: 42,
            },
        }],
        decode: vec![DecodeMeasurement {
            case: DecodeCase {
                name: "C1",
                count: 100,
                comma: false,
                utf8_sentinel: false,
                value_len: 8,
                iterations: 120,
            },
            result: SampleResult {
                ms_per_op: 0.5,
                output_metric: 100,
            },
        }],
    };

    let rendered = render_snapshot_json(&snapshot);
    let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(parsed["encode"][0]["depth"], 2000);
    assert_eq!(parsed["encode"][0]["length"], 42);
    assert_eq!(parsed["decode"][0]["name"], "C1");
    assert_eq!(parsed["decode"][0]["keys"], 100);
}

#[test]
fn text_renderer_handles_single_decode_case_output() {
    let snapshot = Snapshot {
        samples: 3,
        encode: Vec::new(),
        decode: vec![DecodeMeasurement {
            case: DecodeCase {
                name: "C2",
                count: 1000,
                comma: false,
                utf8_sentinel: false,
                value_len: 40,
                iterations: 16,
            },
            result: SampleResult {
                ms_per_op: 0.25,
                output_metric: 1000,
            },
        }],
    };

    let rendered = render_snapshot_text(&snapshot);
    assert!(rendered.contains("C2: count=1000"));
    assert!(!rendered.contains("C1:"));
}

#[test]
fn json_renderer_handles_single_decode_case_output() {
    let snapshot = Snapshot {
        samples: 3,
        encode: Vec::new(),
        decode: vec![DecodeMeasurement {
            case: DecodeCase {
                name: "C3",
                count: 1000,
                comma: true,
                utf8_sentinel: true,
                value_len: 40,
                iterations: 16,
            },
            result: SampleResult {
                ms_per_op: 0.25,
                output_metric: 1000,
            },
        }],
    };

    let rendered = render_snapshot_json(&snapshot);
    let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(parsed["decode"].as_array().unwrap().len(), 1);
    assert_eq!(parsed["decode"][0]["name"], "C3");
    assert!(parsed.get("encode").is_none());
}
