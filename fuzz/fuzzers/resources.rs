#![no_main]
extern crate abxml;
#[macro_use]
extern crate libfuzzer_sys;

use abxml::chunks::ResourceWrapper;

fuzz_target!(|data: &[u8]| {
    let rw = ResourceWrapper::new(data);

    rw.get_resources();
});
