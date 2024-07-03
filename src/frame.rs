use crate::color::Color;
use crate::read_util::ReadUtil;
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct Frame {
    offset: u32,
    table_length: u32,
    length: u32,
}

impl Frame {
    pub fn parse_frames(ref mut data: &[u8], frame_count: u32) -> Result<Vec<Self>> {
        let mut frames = Vec::with_capacity(frame_count as usize);

        for _ in 0..frame_count {
            let frame = Self {
                offset: data.read_u32_le()?,
                table_length: data.read_u32_le()?,
                length: data.read_u32_le()?,
            };

            frames.push(frame);
        }

        Ok(frames)
    }

    pub fn decode(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        palette: &[Color],
    ) -> Result<Vec<u8>> {
        const TILE_WIDTH: usize = 16;
        const TILE_HEIGHT: usize = 16;

        let mut decoded = vec![0; width as usize * height as usize * 4];

        let table = &data[self.offset as usize..];
        let tiles_info = Frame::parse_tile_info(table, self.table_length)?;

        let offset = self.offset as usize + self.table_length as usize;
        let compressed = &data[offset..offset + self.length as usize];

        let mut decompressed = crate::decompress(compressed)?.into_iter();

        for tile_info in tiles_info {
            for x in 0..TILE_WIDTH {
                for y in 0..TILE_HEIGHT {
                    let byte = decompressed.next().context("Unexpected end of stream")?;
                    let index = (byte & 0x1F) as usize;
                    let alpha = ((byte & 0xE0) >> 5) * 36;
                    let color = &palette[index];

                    let pos_y = tile_info.y * TILE_HEIGHT + y;
                    let pos_x = tile_info.x * TILE_WIDTH + x;

                    let pos = (pos_x * width as usize + pos_y) * 4;

                    decoded[pos..pos + 3].copy_from_slice(color.bytes());
                    decoded[pos + 3] = alpha;
                }
            }
        }

        Ok(decoded)
    }

    fn parse_tile_info(ref mut data: &[u8], table_len: u32) -> Result<Vec<TileInfo>> {
        let tile_count = (table_len / 2) as usize;
        let mut tiles_info = Vec::with_capacity(tile_count);

        for _ in 0..tile_count {
            let x = data.read_u8()? as usize;
            let y = data.read_u8()? as usize;

            tiles_info.push(TileInfo { x, y });
        }

        Ok(tiles_info)
    }
}

struct TileInfo {
    x: usize,
    y: usize,
}
