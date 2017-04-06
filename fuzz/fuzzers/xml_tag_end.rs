#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::XmlTagEndWrapper;
use abxml::model::TagEnd;

fuzz_target!(|data: &[u8]| {
    let xtew = XmlTagEndWrapper::new(data);

    xtew.get_id();
});
