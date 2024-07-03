use crate::color::Color;
use crate::read_util::ReadUtil;

use anyhow::{anyhow, Result};

pub struct Header {
    pub num_frames: u32,
    _unknown1: u32,
    pal_len: u32,
    _frame_size: u32,
    _unknown3: u32,
    pub width: u32,
    pub height: u32,
    palette: Vec<Color>,
}

impl Header {
    pub fn parse(ref mut data: &[u8]) -> Result<Self> {
        let num_frames = data.read_u32_le()?;
        let _unknown1 = data.read_u32_le()?;
        let pal_len = data.read_u32_le()?;
        let _frame_size = data.read_u32_le()?;
        let _unknown3 = data.read_u32_le()?;
        let width = data.read_u32_le()?;
        let height = data.read_u32_le()?;
        let palette = Color::parse_palette(&data[..pal_len as usize], pal_len)?;

        if num_frames > 50 {
            return Err(anyhow!("number of frames is not normal"));
        }

        Ok(Self {
            num_frames,
            _unknown1,
            pal_len,
            _frame_size,
            _unknown3,
            width,
            height,
            palette,
        })
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn palette(&self) -> &[Color] {
        &self.palette
    }

    pub fn frame_info_addr(&self) -> usize {
        28 + self.pal_len as usize
    }
}
