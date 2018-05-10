use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use failure::Error;

use chunks::table_type::Region;
use model::Configuration;

#[derive(Clone, Default, Debug)]
pub struct ConfigurationBuf {
    size: u32,
    original_size: u32,
    mcc: u16,
    mnc: u16,
    language: String,
    region: String,
    orientation: u8,
    touchscreen: u8,
    density: u16,
    keyboard: u8,
    navigation: u8,
    input_flags: u8,
    width: u16,
    height: u16,
    sdk_version: u16,
    min_sdk_version: u16,
    screen_layout: u8,
    ui_mode: u8,
    smallest_screen: u16,
    screen_width_dp: u16,
    screen_height_dp: u16,
    locale_script: Option<String>,
    locale_variant: Option<String>,
    secondary_screen_layout: Option<u8>,
}

impl ConfigurationBuf {
    pub fn to_vec(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();

        buffer.write_u32::<LittleEndian>(self.size)?;
        buffer.write_u16::<LittleEndian>(self.mcc)?;
        buffer.write_u16::<LittleEndian>(self.mnc)?;

        let lang = Region::from(self.language.as_ref());
        let (low, high) = lang.into();

        buffer.write_u8(low)?;
        buffer.write_u8(high)?;

        let region = Region::from(self.region.as_ref());
        let (low, high) = region.into();

        buffer.write_u8(low)?;
        buffer.write_u8(high)?;

        buffer.write_u8(self.orientation)?;
        buffer.write_u8(self.touchscreen)?;
        buffer.write_u16::<LittleEndian>(self.density)?;
        buffer.write_u8(self.keyboard)?;
        buffer.write_u8(self.navigation)?;
        buffer.write_u8(self.input_flags)?;
        buffer.write_u8(0)?;

        buffer.write_u16::<LittleEndian>(self.width)?;
        buffer.write_u16::<LittleEndian>(self.height)?;
        buffer.write_u16::<LittleEndian>(self.sdk_version)?;
        buffer.write_u16::<LittleEndian>(self.min_sdk_version)?;

        let current = buffer.len();

        // Fill with 0 up to target size
        for _ in current..self.original_size as usize {
            buffer.write_u8(0)?;
        }

        Ok(buffer)
    }

    pub fn from_cursor(buffer: Vec<u8>) -> Result<Self, Error> {
        let original_size = buffer.len() as u32;
        let mut cursor = Cursor::new(buffer);
        let size = cursor.read_u32::<LittleEndian>()?;
        let mcc = cursor.read_u16::<LittleEndian>()?;
        let mnc = cursor.read_u16::<LittleEndian>()?;

        let lang1 = cursor.read_u8()?;
        let lang2 = cursor.read_u8()?;

        let lang = Region::from((lang1, lang2));
        let str_lang = lang.to_string();

        let reg1 = cursor.read_u8()?;
        let reg2 = cursor.read_u8()?;

        let reg = Region::from((reg1, reg2));
        let str_reg = reg.to_string();

        let orientation = cursor.read_u8()?;
        let touchscreen = cursor.read_u8()?;

        let density = cursor.read_u16::<LittleEndian>()?;

        let keyboard = cursor.read_u8()?;
        let navigation = cursor.read_u8()?;
        let input_flags = cursor.read_u8()?;

        cursor.read_u8()?; // Padding

        let width = cursor.read_u16::<LittleEndian>()?;
        let height = cursor.read_u16::<LittleEndian>()?;
        let sdk_version = cursor.read_u16::<LittleEndian>()?;
        let min_sdk_version = cursor.read_u16::<LittleEndian>()?;

        let mut screen_layout = 0;
        let mut ui_mode = 0;
        let mut smallest_screen = 0;
        let mut screen_width_dp = 0;
        let mut screen_height_dp = 0;

        if size >= 32 {
            screen_layout = cursor.read_u8()?;
            ui_mode = cursor.read_u8()?;
            smallest_screen = cursor.read_u16::<LittleEndian>()?;
        }

        if size >= 36 {
            screen_width_dp = cursor.read_u16::<LittleEndian>()?;
            screen_height_dp = cursor.read_u16::<LittleEndian>()?;
        }

        if size >= 48 {
            // TODO: Read following bytes
            cursor.read_u32::<LittleEndian>()?;
            cursor.read_u32::<LittleEndian>()?;
            cursor.read_u32::<LittleEndian>()?;
        }

        if size >= 52 {
            // TODO: Read bytes
        }

        Ok(Self {
            size,
            original_size,
            mcc,
            mnc,
            language: str_lang,
            region: str_reg,
            orientation,
            touchscreen,
            density,
            keyboard,
            navigation,
            input_flags,
            width,
            height,
            sdk_version,
            min_sdk_version,
            screen_layout,
            ui_mode,
            smallest_screen,
            screen_width_dp,
            screen_height_dp,
            locale_script: None,
            locale_variant: None,
            secondary_screen_layout: None,
        })
    }
}

impl Configuration for ConfigurationBuf {
    fn get_size(&self) -> Result<u32, Error> {
        Ok(self.size)
    }

    fn get_mcc(&self) -> Result<u16, Error> {
        Ok(self.mcc)
    }

    fn get_mnc(&self) -> Result<u16, Error> {
        Ok(self.mnc)
    }

    fn get_language(&self) -> Result<String, Error> {
        Ok(Region::from(self.language.as_ref()).to_string())
    }

    fn get_region(&self) -> Result<String, Error> {
        let region = Region::from(self.region.as_ref());
        Ok(region.to_string())
    }

    fn get_orientation(&self) -> Result<u8, Error> {
        Ok(self.orientation)
    }

    fn get_touchscreen(&self) -> Result<u8, Error> {
        Ok(self.touchscreen)
    }

    fn get_density(&self) -> Result<u16, Error> {
        Ok(self.density)
    }

    fn get_keyboard(&self) -> Result<u8, Error> {
        Ok(self.keyboard)
    }

    fn get_navigation(&self) -> Result<u8, Error> {
        Ok(self.navigation)
    }

    fn get_input_flags(&self) -> Result<u8, Error> {
        Ok(self.input_flags)
    }

    fn get_width(&self) -> Result<u16, Error> {
        Ok(self.width)
    }

    fn get_height(&self) -> Result<u16, Error> {
        Ok(self.height)
    }

    fn get_sdk_version(&self) -> Result<u16, Error> {
        Ok(self.sdk_version)
    }

    fn get_min_sdk_version(&self) -> Result<u16, Error> {
        Ok(self.min_sdk_version)
    }

    fn get_screen_layout(&self) -> Result<u8, Error> {
        Ok(self.screen_layout)
    }

    fn get_ui_mode(&self) -> Result<u8, Error> {
        Ok(self.ui_mode)
    }

    fn get_smallest_screen(&self) -> Result<u16, Error> {
        Ok(self.smallest_screen)
    }

    fn get_screen_width(&self) -> Result<u16, Error> {
        Ok(self.screen_width_dp)
    }

    fn get_screen_height(&self) -> Result<u16, Error> {
        Ok(self.screen_height_dp)
    }

    fn get_locale_script(&self) -> Result<Option<String>, Error> {
        Ok(self.locale_script.clone())
    }

    fn get_locale_variant(&self) -> Result<Option<String>, Error> {
        Ok(self.locale_variant.clone())
    }

    fn get_secondary_layout(&self) -> Result<Option<u8>, Error> {
        Ok(self.secondary_screen_layout)
    }
}

#[cfg(test)]
mod tests {
    use chunks::*;
    use raw_chunks::EXAMPLE_CONFIGURATION;
    use test::compare_chunks;

    #[test]
    fn identity() {
        let owned = ConfigurationWrapper::new(EXAMPLE_CONFIGURATION)
            .to_buffer()
            .unwrap();
        let new_raw = owned.to_vec().unwrap();

        compare_chunks(EXAMPLE_CONFIGURATION, &new_raw);
    }
}
