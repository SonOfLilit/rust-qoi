use byte_order::{ByteOrder, NumberReader};
use core::panic;
use std::{io::BufReader, io::Read};

use derive_new::new;

    #[derive(new)]
pub struct QoiHeader {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub colorspace: u8,
}

#[derive(new)]
pub struct Qoi {
    pub header: QoiHeader,
    pub image: Vec<u8>,
}

pub trait Hash64 {
    fn hash64(&self) -> u8;
}

#[derive(new, Clone, Copy, Debug)]
struct RGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Hash64 for RGBA {
    fn hash64(&self) -> u8 {
        ((self.r as usize * 3 + self.g as usize * 5 + self.b as usize * 7 + self.a as usize * 11)
            % 64) as u8
    }
}

pub fn decode<R: Read>(cursor: R) -> Qoi {
    let buf_reader = BufReader::new(cursor);
    let mut reader = NumberReader::with_order(ByteOrder::BE, buf_reader);
    let header = {
        assert!(reader.read_u8().unwrap() == b'q');
        assert!(reader.read_u8().unwrap() == b'o');
        assert!(reader.read_u8().unwrap() == b'i');
        assert!(reader.read_u8().unwrap() == b'f');
        let header = QoiHeader {
            width: reader.read_u32().unwrap(),
            height: reader.read_u32().unwrap(),
            channels: reader.read_u8().unwrap(),
            colorspace: reader.read_u8().unwrap(),
        };
        assert!(header.colorspace <= 1);
        header
    };
    let mut read8 = || reader.read_u8().unwrap();

    let mut image = vec![0u8; (header.width * header.height * header.channels as u32) as usize];

    let mut memory = [RGBA::new(0, 0, 0, 0); 64];
    let mut prev = RGBA::new(0, 0, 0, 255);
    let mut index = 0;
    let n = (header.width * header.height * header.channels as u32) as usize;
    while index < n {
        let mut decoded = |color: RGBA| {
            image[index] = color.r;
            index += 1;
            image[index] = color.g;
            index += 1;
            image[index] = color.b;
            index += 1;
            if header.channels == 4 {
                image[index] = color.a;
                index += 1;
            }
        };

        let tag = read8();
        let color = match tag {
            0xfe => RGBA::new(read8(), read8(), read8(), prev.a),
            0xff => RGBA::new(read8(), read8(), read8(), read8()),
            _ => match tag >> 6 {
                0 => memory[(tag & 0b111111) as usize],
                1 => {
                    let dr = ((tag >> 4) & 0b11).wrapping_sub(2);
                    let dg = ((tag >> 2) & 0b11).wrapping_sub(2);
                    let db = ((tag >> 0) & 0b11).wrapping_sub(2);
                    RGBA::new(
                        prev.r.wrapping_add(dr),
                        prev.g.wrapping_add(dg),
                        prev.b.wrapping_add(db),
                        prev.a,
                    )
                }
                2 => {
                    let dg = ((tag >> 0) & 0b11111).wrapping_sub(32);
                    assert!(dg >= (256 - 32) as u8 && dg < 32);
                    let byte = read8();
                    let dr_dg = ((byte >> 4) & 0b1111).wrapping_sub(8);
                    let db_dg = ((byte >> 0) & 0b1111).wrapping_sub(8);
                    let g = prev.g.wrapping_sub(dg);
                    let r = dr_dg
                        .wrapping_add(g.wrapping_sub(prev.g))
                        .wrapping_sub(prev.r);
                    let b = db_dg
                        .wrapping_add(g.wrapping_sub(prev.g))
                        .wrapping_sub(prev.b);
                    RGBA::new(r, g, b, prev.a)
                }
                3 => {
                    let length = tag & 0b111111;
                    for _ in 0..length {
                        decoded(prev);
                        continue;
                    }
                    prev
                }
                _ => panic!(),
            },
        };
        decoded(color);
        prev = color;
        memory[color.hash64() as usize] = color;
    }

    Qoi::new(header, image)
}
