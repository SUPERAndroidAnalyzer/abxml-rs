#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::TypeSpecWrapper;
use abxml::model::TypeSpec;

fuzz_target!(|data: &[u8]| {
    let tsw = TypeSpecWrapper::new(data);

    tsw.get_id();
    tsw.get_amount();
    tsw.get_flag(345235); // Replace with random
});
