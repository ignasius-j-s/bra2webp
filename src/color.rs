use crate::read_util::ReadUtil;

use anyhow::{bail, Result};

#[derive(Debug)]
pub struct Color([u8; 3]);

impl Color {
    pub fn parse_palette(ref mut pal: &[u8], pal_len: u32) -> Result<Vec<Self>> {
        let mut palette = vec![];

        if pal_len == 64 {
            let pal_count = pal_len / 2;

            for _ in 0..pal_count {
                let col = pal.read_u16_le()?;
                palette.push(Color::from_15_bits(col));
            }
        } else {
            bail!("unhandled palette length");
        }

        Ok(palette)
    }

    pub fn from_15_bits(col: u16) -> Self {
        let mut b = ((col & 0x7C00) >> 10) * 8;
        let mut g = ((col & 0x3E0) >> 5) * 8;
        let mut r = (col & 0x1F) * 8;

        r = r + r / 32;
        g = g + g / 32;
        b = b + b / 32;

        Self([r as u8, g as u8, b as u8])
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}
