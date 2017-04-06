#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate abxml;

use abxml::chunks::XmlTagStartWrapper;
use abxml::model::TagStart;

fuzz_target!(|data: &[u8]| {
    let xtsw = XmlTagStartWrapper::new(data);

    xtsw.get_line();
    xtsw.get_field1();
    xtsw.get_namespace_index();
    xtsw.get_element_name_index();
    xtsw.get_field2();
    xtsw.get_attributes_amount();
    xtsw.get_class();
    xtsw.get_attribute(1);
    xtsw.get_attribute(19234);
});
