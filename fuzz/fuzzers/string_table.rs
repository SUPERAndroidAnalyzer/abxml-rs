#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::StringTableWrapper;
use abxml::model::StringTable;

fuzz_target!(|data: &[u8]| {
    let st = StringTableWrapper::new(data);

    st.get_flags();
    st.get_strings_len();
    st.get_styles_len();
    st.get_string(3984895); // TODO: Change by random number from `rand` crate

});