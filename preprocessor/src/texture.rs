use std::path::Path;

use core3d::Texture;
use image::{GenericImageView, ImageReader};

// 3DS texture RGBA channels
pub const IDX_R: usize = 3;
pub const IDX_G: usize = 2;
pub const IDX_B: usize = 1;
pub const IDX_A: usize = 0;

// Given an x, y and width of a source texture, get swizzled index into the destination texture
pub fn swizzle(x: u32, y: u32, width: u32) -> usize {
    ((((y >> 3) * (width >> 3) + (x >> 3)) << 6)
        + ((x & 1)
            | ((y & 1) << 1)
            | ((x & 2) << 1)
            | ((y & 2) << 2)
            | ((x & 4) << 2)
            | ((y & 4) << 3))) as usize
}

pub fn load_image<P: AsRef<Path>>(file: P) -> Texture {
    let mut out = Texture {
        data: Vec::new(),
        width: 0,
        height: 0,
    };

    let img = ImageReader::open(file)
        .expect("Coudln't read image file")
        .decode()
        .expect("Couldn't decode image file");

    out.width = img.width() as u16;
    out.height = img.height() as u16;

    for x in 0..img.width() {
        for y in 0..img.height() {
            let dst_idx = swizzle(x, y, img.width());

            let pix = img.get_pixel(x, y);
            out.data[dst_idx * 4 + IDX_R] = pix.0[0];
            out.data[dst_idx * 4 + IDX_G] = pix.0[1];
            out.data[dst_idx * 4 + IDX_B] = pix.0[2];
            out.data[dst_idx * 4 + IDX_A] = pix.0[3];
        }
    }

    out
}
