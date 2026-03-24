use std::time::Instant;

use qs_rust::{DecodeOptions, EncodeOptions, decode, encode};

use super::args::{Args, Scenario};
use super::cases::{DecodeCase, EncodeCase, decode_cases, encode_cases};
use super::payloads::{build_nested, build_query};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SampleResult {
    pub(super) ms_per_op: f64,
    pub(super) output_metric: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct EncodeMeasurement {
    pub(super) case: EncodeCase,
    pub(super) result: SampleResult,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DecodeMeasurement {
    pub(super) case: DecodeCase,
    pub(super) result: SampleResult,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Snapshot {
    pub(super) samples: usize,
    pub(super) encode: Vec<EncodeMeasurement>,
    pub(super) decode: Vec<DecodeMeasurement>,
}

pub(super) fn measure_snapshot(args: &Args) -> Snapshot {
    let encode = if matches!(args.scenario, Scenario::Encode | Scenario::All) {
        encode_cases(args.max_encode_depth)
            .into_iter()
            .map(|case| EncodeMeasurement {
                case,
                result: measure_encode(case, args.warmups, args.samples),
            })
            .collect()
    } else {
        Vec::new()
    };

    let decode = if matches!(args.scenario, Scenario::Decode | Scenario::All) {
        decode_cases(args.decode_case)
            .into_iter()
            .map(|case| DecodeMeasurement {
                case,
                result: measure_decode(case, args.warmups, args.samples),
            })
            .collect()
    } else {
        Vec::new()
    };

    Snapshot {
        samples: args.samples,
        encode,
        decode,
    }
}

fn measure_encode(case: EncodeCase, warmups: usize, samples: usize) -> SampleResult {
    let payload = build_nested(case.depth);
    let options = EncodeOptions::new().with_encode(false);

    for _ in 0..warmups {
        let _ = encode(&payload, &options).expect("encode warmup should succeed");
    }

    let mut timings = Vec::with_capacity(samples);
    let mut output_length = 0usize;
    for _ in 0..samples {
        let start = Instant::now();
        let mut encoded = String::new();
        for _ in 0..case.iterations {
            encoded = encode(&payload, &options).expect("encode measurement should succeed");
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0 / case.iterations as f64;
        output_length = encoded.len();
        timings.push(elapsed);
    }

    SampleResult {
        ms_per_op: median(&mut timings),
        output_metric: output_length,
    }
}

fn measure_decode(case: DecodeCase, warmups: usize, samples: usize) -> SampleResult {
    let query = build_query(case.count, case.comma, case.utf8_sentinel, case.value_len);
    let options = DecodeOptions::new()
        .with_comma(case.comma)
        .with_charset_sentinel(case.utf8_sentinel)
        .with_parameter_limit(usize::MAX)
        .with_parse_lists(true)
        .with_throw_on_limit_exceeded(false);

    for _ in 0..warmups {
        let _ = decode(&query, &options).expect("decode warmup should succeed");
    }

    let mut timings = Vec::with_capacity(samples);
    let mut key_count = 0usize;
    for _ in 0..samples {
        let start = Instant::now();
        let mut decoded = Default::default();
        for _ in 0..case.iterations {
            decoded = decode(&query, &options).expect("decode measurement should succeed");
        }
        let elapsed = start.elapsed().as_secs_f64() * 1000.0 / case.iterations as f64;
        key_count = decoded.len();
        timings.push(elapsed);
    }

    SampleResult {
        ms_per_op: median(&mut timings),
        output_metric: key_count,
    }
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|left, right| left.partial_cmp(right).expect("timings should be finite"));
    values[values.len() / 2]
}
