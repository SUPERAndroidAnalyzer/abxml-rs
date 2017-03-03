use visitor::ModelVisitor;
use std::path::Path;
use std::io::Cursor;
use visitor::Executor;
use errors::*;
use std::io::Read;
use std::io;
use std::fs;
use zip::ZipArchive;
use visitor::*;
use encoder::Xml;
use std::io::Write;
use STR_ARSC;

pub struct Decoder<'a> {
    visitor: ModelVisitor<'a>,
    buffer_android: &'a [u8],
    buffer_apk: &'a [u8],
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let visitor = ModelVisitor::default();

        let mut decoder = Decoder {
            visitor: visitor,
            buffer_android: STR_ARSC,
            buffer_apk: data,
        };

        let android_resources_cursor: Cursor<&[u8]> = Cursor::new(decoder.buffer_android);
        Executor::arsc(android_resources_cursor, &mut decoder.visitor).chain_err(|| "Could not read android lib resources")?;

        let cursor: Cursor<&[u8]> = Cursor::new(decoder.buffer_apk);
        Executor::arsc(cursor, &mut decoder.visitor).chain_err(|| "Could not read target APK resources")?;

        Ok(decoder)
    }

    pub fn get_resources(&self) -> &'a Resources {
        &self.visitor.get_resources()
    }
}