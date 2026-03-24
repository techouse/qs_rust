//! Comma-list encoding helpers.

use crate::key_path::KeyPathNode;
use crate::options::{EncodeOptions, EncodeToken};
use crate::value::Value;

use super::filter::{EncodeInput, apply_filter_result};
use super::scalar::{
    encode_string_or_raw, encoded_scalar_text, filter_prefix, finalize_key_only_fragment,
    finalize_key_path, joined_text_value, plain_string_for_comma, scalar_is_null_like,
};
use super::{array_child_path, ordered_array_indices};

pub(super) fn encode_comma_array(
    items: &[Value],
    path: &KeyPathNode,
    options: &EncodeOptions,
) -> Vec<String> {
    let mut elements = Vec::new();
    let mut encoded_elements = Vec::new();
    for item in items {
        if scalar_is_null_like(item, options) && (options.skip_nulls || options.comma_compact_nulls)
        {
            continue;
        }
        let plain = plain_string_for_comma(item, options);
        if options.encode_values_only {
            encoded_elements.push(encode_comma_value_item(item, &plain, options));
        }
        elements.push(plain);
    }

    if options.comma_compact_nulls && elements.is_empty() {
        return Vec::new();
    }

    if elements.is_empty() {
        if items.is_empty() {
            if options.allow_empty_lists {
                let key_path = path.append_empty_list_suffix();
                return vec![finalize_key_path(key_path.materialize(), options)];
            }
            return Vec::new();
        }
        if options.strict_null_handling {
            let key_path = if options.comma_round_trip && items.len() == 1 {
                path.append_empty_list_suffix()
            } else {
                path.clone()
            };
            return vec![finalize_key_only_fragment(key_path.materialize(), options)];
        }
    }

    let key_path = if options.comma_round_trip && elements.len() == 1 {
        path.append_empty_list_suffix()
    } else {
        path.clone()
    };
    let key = finalize_key_path(key_path.materialize(), options);
    let joined = elements.join(",");
    let value = if options.encode_values_only {
        encoded_elements.join(",")
    } else {
        encode_string_or_raw(&joined_text_value(&joined, options), options)
    };
    vec![format!("{key}={value}")]
}

pub(super) fn encode_comma_array_controlled(
    items: &[Value],
    path: &KeyPathNode,
    options: &EncodeOptions,
) -> Vec<String> {
    let indices = ordered_array_indices(items, options);

    if items.is_empty() {
        if options.allow_empty_lists {
            let key_path = path.append_empty_list_suffix();
            return vec![finalize_key_path(key_path.materialize(), options)];
        }
        return Vec::new();
    }

    let mut kept_items = 0usize;
    let mut elements = Vec::new();
    let mut encoded_elements = Vec::new();

    for index in indices {
        let Some(item) = items.get(index) else {
            continue;
        };

        let item_path = array_child_path(path, index, options);
        let prefix = filter_prefix(item_path.materialize(), options);
        let input = apply_filter_result(EncodeInput::Borrowed(item), &prefix, options);
        let Some(value) = input.as_value() else {
            continue;
        };

        kept_items += 1;
        if scalar_is_null_like(value, options)
            && (options.skip_nulls || options.comma_compact_nulls)
        {
            continue;
        }

        let plain = plain_string_for_comma(value, options);
        if options.encode_values_only {
            encoded_elements.push(encode_comma_value_item(value, &plain, options));
        }
        elements.push(plain);
    }

    if kept_items == 0 {
        return Vec::new();
    }

    if options.comma_compact_nulls && elements.is_empty() {
        return Vec::new();
    }

    if elements.is_empty() && options.strict_null_handling {
        let key_path = if options.comma_round_trip && kept_items == 1 {
            path.append_empty_list_suffix()
        } else {
            path.clone()
        };
        return vec![finalize_key_only_fragment(key_path.materialize(), options)];
    }

    let key_path = if options.comma_round_trip && elements.len() == 1 {
        path.append_empty_list_suffix()
    } else {
        path.clone()
    };
    let key = finalize_key_path(key_path.materialize(), options);
    let joined = elements.join(",");
    let value = if options.encode_values_only {
        encoded_elements.join(",")
    } else {
        encode_string_or_raw(&joined_text_value(&joined, options), options)
    };
    vec![format!("{key}={value}")]
}

fn encode_comma_value_item(value: &Value, plain: &str, options: &EncodeOptions) -> String {
    if matches!(value, Value::Temporal(_)) {
        return encode_string_or_raw(
            &encoded_scalar_text(value, options)
                .expect("temporal comma item should resolve before encoding"),
            options,
        );
    }

    if let Some(encoder) = options.encoder.as_ref() {
        return encode_string_or_raw(
            &encoder.encode(EncodeToken::Value(value), options.charset, options.format),
            options,
        );
    }

    encode_string_or_raw(plain, options)
}
