use crate::model::{owned::ConfigurationBuf, Configuration};
use anyhow::{bail, ensure, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{io::Cursor, string::ToString};

#[derive(Debug)]
pub struct ConfigurationWrapper<'a> {
    slice: &'a [u8],
}

impl<'a> ConfigurationWrapper<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice }
    }

    pub fn to_buffer(&self) -> Result<ConfigurationBuf> {
        ConfigurationBuf::from_cursor(self.slice.into())
    }
}

impl<'a> Configuration for ConfigurationWrapper<'a> {
    fn size(&self) -> Result<u32> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(0);

        Ok(cursor.read_u32::<LittleEndian>()?)
    }

    fn mcc(&self) -> Result<u16> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(4);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn mnc(&self) -> Result<u16> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(6);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn language(&self) -> Result<String> {
        let lang_low = self.slice[8];
        let lang_high = self.slice[9];

        let region = Region::from((lang_low, lang_high));

        Ok(region.to_string())
    }

    fn region(&self) -> Result<String> {
        let lang_low = self.slice[10];
        let lang_high = self.slice[11];

        let region = Region::from((lang_low, lang_high));

        Ok(region.to_string())
    }

    fn orientation(&self) -> Result<u8> {
        Ok(self.slice[12])
    }

    fn touchscreen(&self) -> Result<u8> {
        Ok(self.slice[13])
    }

    fn density(&self) -> Result<u16> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(14);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn keyboard(&self) -> Result<u8> {
        Ok(self.slice[16])
    }

    fn navigation(&self) -> Result<u8> {
        Ok(self.slice[17])
    }

    fn input_flags(&self) -> Result<u8> {
        Ok(self.slice[18])
    }

    fn width(&self) -> Result<u16> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(20);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn height(&self) -> Result<u16> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(22);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn sdk_version(&self) -> Result<u16> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(24);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn min_sdk_version(&self) -> Result<u16> {
        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(26);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn screen_layout(&self) -> Result<u8> {
        let size = self.size()?;
        ensure!(size >= 28, "not enough bytes to retrieve the field");

        Ok(self.slice[28])
    }

    fn ui_mode(&self) -> Result<u8> {
        let size = self.size()?;
        ensure!(size >= 29, "not enough bytes to retrieve the field");

        Ok(self.slice[29])
    }

    fn smallest_screen(&self) -> Result<u16> {
        let size = self.size()?;
        ensure!(size >= 30, "not enough bytes to retrieve the field");

        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(30);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn screen_width(&self) -> Result<u16> {
        let size = self.size()?;
        ensure!(size >= 32, "not enough bytes to retrieve the field");

        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(32);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn screen_height(&self) -> Result<u16> {
        let size = self.size()?;
        ensure!(size >= 34, "not enough bytes to retrieve the field");

        let mut cursor = Cursor::new(self.slice);
        cursor.set_position(34);

        Ok(cursor.read_u16::<LittleEndian>()?)
    }

    fn locale_script(&self) -> Result<Option<String>> {
        bail!("not enough bytes to retrieve the field")
    }

    fn locale_variant(&self) -> Result<Option<String>> {
        bail!("not enough bytes to retrieve the field")
    }

    fn secondary_layout(&self) -> Result<Option<u8>> {
        bail!("not enough bytes to retrieve the field")
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Region {
    low: u8,
    high: u8,
}

impl Into<(u8, u8)> for Region {
    fn into(self) -> (u8, u8) {
        (self.low, self.high)
    }
}

impl<'a> From<&'a [u8]> for Region {
    fn from(input: &'a [u8]) -> Self {
        if let [low, high] = *input {
            Self { low, high }
        } else {
            Self::default()
        }
    }
}

impl From<(u8, u8)> for Region {
    fn from(input: (u8, u8)) -> Self {
        Self {
            low: input.0,
            high: input.1,
        }
    }
}

impl ToString for Region {
    fn to_string(&self) -> String {
        let mut chrs = Vec::new();

        if self.low == 0 && self.high == 0 {
            return "any".to_owned();
        }

        chrs.push(self.low);
        chrs.push(self.high);

        String::from_utf8(chrs).unwrap_or_else(|_| String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::{Configuration, ConfigurationWrapper, Region, ToString};
    use crate::raw_chunks::EXAMPLE_CONFIGURATION;

    #[test]
    fn it_can_encode_bytes_region_to_string() {
        let region = Region::from((99, 97));

        assert_eq!("ca", region.to_string());
    }

    #[test]
    fn it_can_encode_bytes_region_to_string_any() {
        let region = Region::from((0, 0));

        assert_eq!("any", region.to_string());
    }

    #[test]
    fn it_can_encode_bytes_region_from_string() {
        let region = Region::from("ps".as_ref());
        let (low, high) = region.into();

        assert_eq!(112, low);
        assert_eq!(115, high);
    }

    #[test]
    fn it_can_encode_bytes_region_from_string_any() {
        let region = Region::from("any".as_ref());
        let (low, high) = region.into();

        assert_eq!(0, low);
        assert_eq!(0, high);
    }

    #[test]
    fn it_can_decode_a_full_configuration_slice() {
        let wrapper = ConfigurationWrapper::new(EXAMPLE_CONFIGURATION);

        assert_eq!(56, wrapper.get_size().unwrap());
        assert_eq!(310, wrapper.get_mcc().unwrap());
        assert_eq!(800, wrapper.get_mnc().unwrap());
        assert_eq!("bs", wrapper.get_language().unwrap());
        assert_eq!("BA", wrapper.get_region().unwrap());
        assert_eq!(0, wrapper.get_orientation().unwrap());
        assert_eq!(0, wrapper.get_touchscreen().unwrap());
        assert_eq!(0, wrapper.get_density().unwrap());
        assert_eq!(0, wrapper.get_keyboard().unwrap());
        assert_eq!(0, wrapper.get_keyboard().unwrap());
        assert_eq!(0, wrapper.get_width().unwrap());
        assert_eq!(0, wrapper.get_height().unwrap());
    }
}
