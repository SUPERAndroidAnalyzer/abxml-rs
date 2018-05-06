#![no_main]
extern crate abxml;
#[macro_use]
extern crate libfuzzer_sys;

use abxml::chunks::XmlTextWrapper;

fuzz_target!(|data: &[u8]| {
    let xtw = XmlTextWrapper::new(data);

    xtw.get_text_index();
});
