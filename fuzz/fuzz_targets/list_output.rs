#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    envq_fuzz::run_list_output_bytes(data);
});
