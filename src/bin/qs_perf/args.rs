use std::env;

use super::cases::{DecodeCase, parse_decode_case};

const DEFAULT_WARMUPS: usize = 5;
const DEFAULT_SAMPLES: usize = 7;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Scenario {
    Encode,
    Decode,
    All,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Args {
    pub(super) scenario: Scenario,
    pub(super) format: OutputFormat,
    pub(super) warmups: usize,
    pub(super) samples: usize,
    pub(super) decode_case: Option<DecodeCase>,
    pub(super) max_encode_depth: Option<usize>,
    pub(super) output: Option<String>,
}

pub(super) fn parse_args() -> Args {
    parse_args_from(env::args().skip(1))
}

pub(super) fn parse_args_from<I, S>(args: I) -> Args
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut scenario = Scenario::All;
    let mut format = OutputFormat::Text;
    let mut warmups = DEFAULT_WARMUPS;
    let mut samples = DEFAULT_SAMPLES;
    let mut decode_case = None;
    let mut max_encode_depth = None;
    let mut output = None;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "--scenario" => {
                let value = args.next().expect("--scenario requires a value");
                scenario = match value.as_ref() {
                    "encode" => Scenario::Encode,
                    "decode" => Scenario::Decode,
                    "all" => Scenario::All,
                    _ => panic!("unsupported scenario: {}", value.as_ref()),
                };
            }
            "--format" => {
                let value = args.next().expect("--format requires a value");
                format = match value.as_ref() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    _ => panic!("unsupported format: {}", value.as_ref()),
                };
            }
            "--warmups" => {
                warmups = args
                    .next()
                    .expect("--warmups requires a value")
                    .as_ref()
                    .parse()
                    .expect("--warmups must be an integer");
            }
            "--samples" => {
                samples = args
                    .next()
                    .expect("--samples requires a value")
                    .as_ref()
                    .parse()
                    .expect("--samples must be an integer");
            }
            "--decode-case" => {
                let value = args.next().expect("--decode-case requires a value");
                decode_case = Some(parse_decode_case(value.as_ref()));
            }
            "--max-encode-depth" => {
                max_encode_depth = Some(
                    args.next()
                        .expect("--max-encode-depth requires a value")
                        .as_ref()
                        .parse()
                        .expect("--max-encode-depth must be an integer"),
                );
            }
            "--output" => {
                output = Some(
                    args.next()
                        .expect("--output requires a path")
                        .as_ref()
                        .to_owned(),
                );
            }
            _ => panic!("unsupported argument: {}", arg.as_ref()),
        }
    }

    assert!(samples > 0, "--samples must be > 0");

    Args {
        scenario,
        format,
        warmups,
        samples,
        decode_case,
        max_encode_depth,
        output,
    }
}

#[cfg(test)]
mod tests {
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
}
