// https://qoiformat.org/qoi-specification.pdf decoder by Aur Saraf

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

use qoi::{decode, Qoi};

fn main() {
    if env::args().len() != 3 {
        panic!("usage: qoi path/to/source.qoi path/to/dest.png");
    }
    let source_path = env::args().nth(1).unwrap();
    let dest_path = env::args().nth(2).unwrap();
    let file = File::open(source_path).expect("error opening file");
    let image = decode(file);

    let file = File::create(dest_path).expect("error creating output file");
    encode(file, image);
}

fn encode<W: Write>(cursor: W, image: Qoi) {
    let ref mut w = BufWriter::new(cursor);
    let mut encoder = png::Encoder::new(w, image.header.width, image.header.height);
    let color_type = match image.header.channels {
        3 => png::ColorType::Rgb,
        4 => png::ColorType::Rgba,
        _ => panic!("must have 3 or 4 channels"),
    };
    encoder.set_color(color_type);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&image.image).unwrap();
}
