pub mod bra;
pub mod color;
pub mod frame;
pub mod read_util;

use std::io::{Cursor, Seek};

use frame::Frame;
use read_util::ReadUtil;

use anyhow::{bail, Context, Result};
use bitvec::prelude::*;
use rgb::FromSlice;
use webp_animation::{
    ColorMode::Rgba, Encoder, EncoderOptions, EncodingConfig, EncodingType::Lossless, WebPData,
};

pub fn decompress(src: &[u8]) -> Result<Vec<u8>> {
    let mut comp = Cursor::new(src);
    let mut decomp = vec![];

    let mut read_flag = true;
    let mut method = 0;

    'outer: while comp.stream_position()? < src.len() as u64 {
        let mut start: usize = 0;
        let byte = comp.read_u8()?;
        let bitv = BitVec::<_, Msb0>::from_element(byte);

        if read_flag {
            start = 4;
            method = byte >> 4;
            read_flag = false;
        }

        for i in start..8 {
            if bitv[i] {
                let val = comp.read_u8()?;
                decomp.push(val);
            } else {
                let val1 = comp.read_u8()?;
                let val2 = comp.read_u8()?;

                if (val1, val2) == (0xff, 0xff) {
                    read_flag = true;
                    continue 'outer;
                }

                let length;
                let offset;

                match method {
                    0x4 => {
                        length = (val2 >> 3) + 2;
                        offset = 0xf800 + ((val2 as u16 & 0x7) << 8) + val1 as u16;
                    }
                    0x8 => {
                        length = (val2 >> 4) + 2;
                        offset = 0xf000 + ((val2 as u16 & 0xf) << 8) + val1 as u16;
                    }
                    0xC => {
                        length = (val2 >> 5) + 2;
                        offset = 0xe000 + ((val2 as u16 & 0x1f) << 8) + val1 as u16;
                    }
                    0x0 => {
                        length = (val2 >> 6) + 2;
                        offset = 0xc000 + ((val2 as u16 & 0x3f) << 8) + val1 as u16;
                    }
                    other => bail!("Unknown method flag: {}", other),
                }

                let offset = i16::from_be_bytes(offset.to_be_bytes());

                if offset.is_positive() || offset == 0 {
                    let byte = decomp[offset as usize];
                    decomp.extend(vec![byte; length as usize]);
                } else if decomp.len().wrapping_add_signed(offset as isize) == 0 {
                    decomp.extend(vec![0; length as usize]);
                } else {
                    for _ in 0..length {
                        let index = decomp.len().wrapping_add_signed(offset as isize);
                        decomp.push(decomp[index])
                    }
                }
            }
        }
    }

    Ok(decomp)
}

pub fn encode_anim(
    data: &[u8],
    frames: Vec<Frame>,
    frame_duration: i32,
    info: bra::Header,
) -> Result<WebPData> {
    let mut encoder = Encoder::new(info.dimensions()).context("fail to init encoder")?;

    for (i, frame) in frames.iter().enumerate() {
        let image = frame.decode(&data, info.width, info.height, &info.palette())?;

        encoder
            .add_frame(&image, i as i32 * frame_duration)
            .context("fail to add frame")?;
    }

    encoder
        .finalize(frames.len() as i32 * frame_duration)
        .context("failed when finalize encoder")
}

pub fn encode_sticker(
    data: &[u8],
    frames: Vec<Frame>,
    frame_duration: i32,
    info: bra::Header,
) -> Result<WebPData> {
    let options = EncoderOptions {
        anim_params: Default::default(),
        minimize_size: false,
        kmin: 0,
        kmax: 0,
        allow_mixed: false,
        verbose: false,
        color_mode: Rgba,
        encoding_config: Some(EncodingConfig {
            encoding_type: Lossless,
            quality: 70.0,
            method: 0,
        }),
    };

    let mut encoder =
        Encoder::new_with_options((512, 512), options).context("fail to init encoder")?;

    for (i, frame) in frames.iter().enumerate() {
        let image = frame.decode(&data, info.width, info.height, &info.palette())?;
        let source = enlarge_canvas(image, info.width as usize, info.height as usize, 256, 256)?;
        let mut dest = vec![0_u8; 512 * 512 * 4];

        let mut resizer = resize::new(
            256,
            256,
            512,
            512,
            resize::Pixel::RGBA8,
            resize::Type::Point,
        )
        .context("failed to resize")?;

        resizer
            .resize(source.as_rgba(), dest.as_rgba_mut())
            .context("failed to resize")?;

        encoder
            .add_frame(&dest, i as i32 * frame_duration)
            .context("fail to add frame")?;
    }

    encoder
        .finalize(frames.len() as i32 * frame_duration)
        .context("failed when finalize encoder")
}

fn enlarge_canvas(
    image: Vec<u8>,
    width: usize,
    height: usize,
    nwidth: usize,
    nheight: usize,
) -> Result<Vec<u8>> {
    if nwidth < width || height > nheight {
        bail!("new dimensions is smaller");
    }

    let mut resized = vec![0; nwidth * nheight * 4];
    let left_offset = (nwidth - width) / 2;
    let top_offset = nheight - height;

    for y in 0..height {
        for x in 0..width {
            let index1 = (y * width + x) * 4;
            let index2 = ((top_offset + y) * nwidth + (left_offset + x)) * 4;

            resized[index2 + 0] = image[index1 + 0];
            resized[index2 + 1] = image[index1 + 1];
            resized[index2 + 2] = image[index1 + 2];
            resized[index2 + 3] = image[index1 + 3];
        }
    }

    Ok(resized)
}
