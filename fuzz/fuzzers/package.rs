#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::PackageWrapper;

fuzz_target!(|data: &[u8]| {
    let pw = PackageWrapper::new(data);
    pw.get_id();
    pw.get_name();
});
