#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    jd_fuzz::fuzz_canonicalization(data);
});
