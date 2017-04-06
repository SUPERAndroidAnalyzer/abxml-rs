#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::TableTypeWrapper;
use abxml::model::TableType;

fuzz_target!(|data: &[u8]| {
    let ttw = TableTypeWrapper::new(data, 68);

    ttw.get_id();
    ttw.get_amount();
    ttw.get_configuration();
    ttw.get_entry(213514); // Replace with random
    ttw.get_entry(1);
    ttw.get_entries();
});
