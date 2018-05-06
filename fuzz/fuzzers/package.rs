#![no_main]
extern crate abxml;
#[macro_use]
extern crate libfuzzer_sys;

use abxml::chunks::PackageWrapper;

fuzz_target!(|data: &[u8]| {
    let pw = PackageWrapper::new(data);
    pw.get_id();
    pw.get_name();
});
