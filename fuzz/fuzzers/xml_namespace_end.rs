#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::XmlNamespaceEndWrapper;
use abxml::model::NamespaceEnd;

fuzz_target!(|data: &[u8]| {
    let xnew = XmlNamespaceEndWrapper::new(data);

    xnew.get_prefix_index();
    xnew.get_namespace_index();
    xnew.get_line();
});