extern crate abxml;

use abxml::decoder::Decoder;
use abxml::model::builder::Xml;
use abxml::model::owned::{AttributeBuf, StringTableBuf, XmlTagEndBuf, XmlTagStartBuf};

#[test]
fn it_can_generate_a_decoder_from_a_buffer() {
    let arsc = vec![2, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut xml = Xml::default();
    let mut st = StringTableBuf::default();
    st.add_string("Some string".to_string());
    st.add_string("Another srtring".to_string());
    st.add_string("start_tag".to_string());
    st.add_string("key".to_string());
    st.add_string("value".to_string());

    let attribute = AttributeBuf::new(0xFFFF_FFFF, 3, 0xFFFF_FFFF, 3 << 24, 4);

    let mut tag_start = XmlTagStartBuf::new(2, 0, 0xFFFF_FFFF, 2, 0, 0);
    tag_start.add_attribute(attribute);

    xml.push_owned(Box::new(st));
    xml.push_owned(Box::new(tag_start));
    xml.push_owned(Box::new(XmlTagEndBuf::new(90)));

    let xml_content = xml.into_vec().unwrap();
    let decoder = Decoder::from_buffer(&arsc).unwrap();
    let xml_visitor = decoder.xml_visitor(&xml_content).unwrap();
    let out = xml_visitor.into_string().unwrap();

    let inner_expected = "<start_tag key=\"value\" />";
    let expected = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n{}",
        inner_expected
    );
    assert_eq!(expected, out);
}
