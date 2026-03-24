#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EncodeCase {
    pub(super) depth: usize,
    pub(super) iterations: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct DecodeCase {
    pub(super) name: &'static str,
    pub(super) count: usize,
    pub(super) comma: bool,
    pub(super) utf8_sentinel: bool,
    pub(super) value_len: usize,
    pub(super) iterations: usize,
}

const ENCODE_CASES: [EncodeCase; 3] = [
    EncodeCase {
        depth: 2000,
        iterations: 20,
    },
    EncodeCase {
        depth: 5000,
        iterations: 20,
    },
    EncodeCase {
        depth: 12000,
        iterations: 8,
    },
];

const DECODE_CASES: [DecodeCase; 3] = [
    DecodeCase {
        name: "C1",
        count: 100,
        comma: false,
        utf8_sentinel: false,
        value_len: 8,
        iterations: 120,
    },
    DecodeCase {
        name: "C2",
        count: 1000,
        comma: false,
        utf8_sentinel: false,
        value_len: 40,
        iterations: 16,
    },
    DecodeCase {
        name: "C3",
        count: 1000,
        comma: true,
        utf8_sentinel: true,
        value_len: 40,
        iterations: 16,
    },
];

pub(super) fn parse_decode_case(value: &str) -> DecodeCase {
    DECODE_CASES
        .into_iter()
        .find(|case| case.name == value)
        .unwrap_or_else(|| panic!("unsupported decode case: {value}"))
}

pub(super) fn encode_cases(max_depth: Option<usize>) -> Vec<EncodeCase> {
    ENCODE_CASES
        .into_iter()
        .filter(|case| max_depth.is_none_or(|limit| case.depth <= limit))
        .collect()
}

pub(super) fn decode_cases(filter: Option<DecodeCase>) -> Vec<DecodeCase> {
    DECODE_CASES
        .into_iter()
        .filter(|case| filter.is_none_or(|wanted| wanted == *case))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{DecodeCase, EncodeCase, decode_cases, encode_cases};

    #[test]
    fn encode_case_filter_applies_max_depth_cap() {
        let cases = encode_cases(Some(4000));
        assert_eq!(
            cases,
            vec![EncodeCase {
                depth: 2000,
                iterations: 20,
            }]
        );
    }

    #[test]
    fn decode_case_filter_limits_available_cases() {
        let cases = decode_cases(Some(DecodeCase {
            name: "C2",
            count: 1000,
            comma: false,
            utf8_sentinel: false,
            value_len: 40,
            iterations: 16,
        }));

        assert_eq!(
            cases,
            vec![DecodeCase {
                name: "C2",
                count: 1000,
                comma: false,
                utf8_sentinel: false,
                value_len: 40,
                iterations: 16,
            }]
        );
    }
}
