//! Flat decode accumulation façade.

mod build;
mod combine;
mod insert;
mod process;

pub(super) use self::combine::combine_with_limit;
pub(super) use self::insert::insert_value;
pub(super) use self::process::{
    process_plain_part_default, process_query_part_custom, process_query_part_default,
    process_scanned_part_custom, process_scanned_part_default_accumulator,
};
