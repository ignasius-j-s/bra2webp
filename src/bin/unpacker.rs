use std::{env::args, fs::create_dir, path::Path, process::exit};

use bra2webp::read_util::ReadUtil;

fn main() {
    let args = args().collect::<Vec<_>>();

    let input_files = args
        .into_iter()
        .filter(|arg| arg.ends_with(".pack"))
        .collect::<Vec<_>>();

    if input_files.is_empty() {
        print_help();
        exit(1);
    }

    for path in input_files {
        let Ok(mut file) = std::fs::File::open(&path) else {
            println!("can't open {}", &path);
            continue;
        };

        if file.read_u32_be().is_ok_and(|sig| sig == 0) {
            unpack(file, &path);
        } else {
            println!("{} isn't a valid pack file", &path);
            continue;
        }
    }
}

fn unpack(mut file: std::fs::File, path: &str) {
    use std::io::prelude::*;
    use std::io::SeekFrom::*;

    let path = Path::new(&path).with_extension("");

    if !path.is_dir() {
        create_dir(&path).unwrap();
    }

    file.seek(Start(4)).unwrap();

    let num_files = file.read_u32_be().unwrap();
    let offset = file.read_u32_be().unwrap() + 4;
    let mut offset = offset as usize;

    file.seek(Start(0x10)).unwrap();

    let mut entries = Vec::with_capacity(num_files as usize);

    for _ in 0..num_files {
        let name = file.read_pascal_string().unwrap();
        let length = file.read_u32_be().unwrap() as usize;

        entries.push((offset as u64, length, name));
        offset += length;
    }

    for entry in entries {
        let mut buf = vec![0; entry.1];
        file.seek(Start(entry.0)).unwrap();
        file.read_exact(&mut buf).unwrap();

        let output = path.join(entry.2);
        std::fs::write(output, buf).unwrap();
    }
}

fn print_help() {
    println!("usage: unpacker pack_file(s) ...")
}
