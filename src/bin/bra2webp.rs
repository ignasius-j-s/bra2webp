use std::{path::Path, process::exit};

use bra2webp::{bra::Header, frame::Frame};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut frame_duration: i32 = 150;

    if let Some(index) = args.iter().position(|arg| arg == "-d") {
        if let Some(Ok(duration)) = args.get(index + 1).map(|arg| arg.parse()) {
            frame_duration = duration;
        };
    };

    let sticker = args.iter().any(|arg| arg == "-s");

    let input_files: Vec<String> = args
        .into_iter()
        .filter(|arg| arg.ends_with(".bra"))
        .collect();

    if input_files.is_empty() {
        print_help();
        exit(1);
    }

    for file in input_files {
        if let Ok(data) = std::fs::read(&file) {
            let info = Header::parse(&data).context("invalid bra file")?;
            let frames = Frame::parse_frames(&data[info.frame_info_addr()..], info.num_frames)?;
            let output = Path::new(&file).with_extension("webp");
            let webp_data = if sticker {
                bra2webp::encode_sticker(&data, frames, frame_duration, info)?
            } else {
                bra2webp::encode_anim(&data, frames, frame_duration, info)?
            };

            match std::fs::write(&output, &webp_data) {
                Ok(_) => println!(
                    "{} is successfully converted into {}",
                    &file,
                    output.display()
                ),
                Err(_) => println!("failed to convert: {}", output.display()),
            }
        } else {
            print!("can't read this file: {file}");
            continue;
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: bra2webp [-switches] bra_file(s) ...");
    println!("Switches:");
    println!("  -d [dur]  Duration per frame in ms. Default is 150");
    println!("  -s        make output suitable for WhatsApp Sticker");
}
