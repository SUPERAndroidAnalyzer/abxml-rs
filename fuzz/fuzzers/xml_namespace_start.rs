#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::XmlNamespaceStartWrapper;
use abxml::model::NamespaceStart;

fuzz_target!(|data: &[u8]| {
    let xnsw = XmlNamespaceStartWrapper::new(data);

    xnsw.get_prefix_index();
    xnsw.get_namespace_index();
    xnsw.get_line();
});
