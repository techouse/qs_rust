#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    qs_rust_fuzz::run_decode_pairs_bytes(data);
});
