extern crate abxml;

use std::path::Path;
use abxml::decoder::{Apk, Decoder};
use abxml::model::builder::Xml;
use abxml::model::owned::{XmlTagStartBuf, XmlTagEndBuf, StringTableBuf};
use std::borrow::Cow;

#[test]
#[should_panic]
fn it_can_generate_a_decoder_from_an_apk() {
    let path = Path::new("some.apk");
    Apk::new(path).unwrap();
}

#[test]
fn it_can_generate_a_decoder_from_a_buffer() {
    let arsc = vec![0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut xml = Xml::default();
    let mut st = StringTableBuf::default();
    st.add_string("Some string".to_string());
    st.add_string("Another srtring".to_string());
    st.add_string("start_tag".to_string());

    xml.push_owned(Box::new(st));
    xml.push_owned(Box::new(XmlTagStartBuf::new(2, None)));
    xml.push_owned(Box::new(XmlTagEndBuf::new(90)));

    let xml_content = xml.into_vec().unwrap();
    let decoder = Decoder::new(&arsc).unwrap();
    let out = decoder.as_xml(&xml_content).unwrap();

    let inner_expected = "<start_tag></start_tag>";
    let expected = format!("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"no\"?>\n{}",
                           inner_expected);

    assert_eq!(expected, out);
}
