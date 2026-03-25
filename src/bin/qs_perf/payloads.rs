use qs_rust::Value;

pub(super) fn build_nested(depth: usize) -> Value {
    let mut current = Value::Object([("leaf".to_owned(), Value::String("x".to_owned()))].into());
    for _ in 0..depth {
        current = Value::Object([("a".to_owned(), current)].into());
    }
    current
}

fn make_value(length: usize, seed: usize) -> String {
    let mut out = String::with_capacity(length);
    let mut state = ((seed as u32).wrapping_mul(2_654_435_761)).wrapping_add(1_013_904_223);
    for _ in 0..length {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;

        let x = (state % 62) as u8;
        let ch = if x < 10 {
            (b'0' + x) as char
        } else if x < 36 {
            (b'A' + (x - 10)) as char
        } else {
            (b'a' + (x - 36)) as char
        };
        out.push(ch);
    }
    out
}

pub(super) fn build_query(
    count: usize,
    comma_lists: bool,
    utf8_sentinel: bool,
    value_len: usize,
) -> String {
    let mut parts = Vec::with_capacity(count + usize::from(utf8_sentinel));
    if utf8_sentinel {
        parts.push("utf8=%E2%9C%93".to_owned());
    }

    for index in 0..count {
        let value = if comma_lists && index % 10 == 0 {
            "a,b,c".to_owned()
        } else {
            make_value(value_len, index)
        };
        parts.push(format!("k{index}={value}"));
    }

    parts.join("&")
}

#[cfg(test)]
mod tests;
