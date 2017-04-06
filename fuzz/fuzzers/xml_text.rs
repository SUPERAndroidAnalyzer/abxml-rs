#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::XmlTextWrapper;

fuzz_target!(|data: &[u8]| {
    let xtw = XmlTextWrapper::new(data);

    xtw.get_text_index();
});
