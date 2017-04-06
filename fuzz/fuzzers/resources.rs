#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::ResourceWrapper;

fuzz_target!(|data: &[u8]| {
    let rw = ResourceWrapper::new(data);

    rw.get_resources();
});
