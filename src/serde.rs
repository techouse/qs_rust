//! Serde integration helpers.

mod bridge;
mod deserializer;
mod serializer;
pub mod temporal;

pub use bridge::{from_str, from_value, to_string, to_value};

#[cfg(test)]
mod tests;
