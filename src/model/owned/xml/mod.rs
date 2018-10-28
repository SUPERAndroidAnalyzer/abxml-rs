mod attribute;
mod namespace_end;
mod namespace_start;
mod tag_end;
mod tag_start;

pub use crate::model::owned::xml::{
    attribute::AttributeBuf, namespace_end::XmlNamespaceEndBuf,
    namespace_start::XmlNamespaceStartBuf, tag_end::XmlTagEndBuf, tag_start::XmlTagStartBuf,
};
