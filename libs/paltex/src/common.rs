use std::fmt::{Debug, Display};

use bang_core::draw::AsBytes;

#[derive(Debug)]
pub struct InvalidInput;

impl Display for InvalidInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct Header {
    pub width: u16,
    pub height: u16,
    pub pal_len: u8,
    pub padding: u8,
}

unsafe impl AsBytes for Header {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(4))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

unsafe impl AsBytes for Color {}

impl Color {
    pub const TRANSPARENT: Color = Color::from_rgba_u8([0, 0, 0, 0]);

    pub const fn from_rgba_f32(rgba: [f32; 4]) -> Self {
        let max = u8::MAX as f32;
        assert!(rgba[0] >= 0.0 && rgba[0] <= 1.0);
        assert!(rgba[1] >= 0.0 && rgba[1] <= 1.0);
        assert!(rgba[2] >= 0.0 && rgba[2] <= 1.0);
        assert!(rgba[3] >= 0.0 && rgba[3] <= 1.0);
        Self {
            r: (rgba[0] * max) as u8,
            g: (rgba[1] * max) as u8,
            b: (rgba[2] * max) as u8,
            a: (rgba[3] * max) as u8,
        }
    }

    pub const fn from_rgba_u8(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    pub const fn to_rgba_u8(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

pub struct PalTex {
    pub width: u32,
    pub height: u32,
    pub palette: Vec<Color>,
    pub data: Vec<u8>,
}

impl PalTex {
    pub fn from_encoded(bytes: &[u8]) -> Result<Self, InvalidInput> {
        crate::decode(bytes)
    }

    pub fn from_ascii_map<const N: usize, const M: usize>(
        color_legend: &[(u8, Color)],
        bytes: &[[u8; N]; M],
    ) -> Self {
        let mut data = bytes.as_flattened().to_owned();
        let mut color_idx_lookup = [0_u8; 256];
        let mut palette = Vec::with_capacity(color_legend.len());
        palette.push(Color::TRANSPARENT);

        // Prepare the palette and the lookup table
        for (old_idx, color) in color_legend.iter() {
            if *color == Color::TRANSPARENT {
                color_idx_lookup[*old_idx as usize] = 0;
                continue;
            }
            color_idx_lookup[*old_idx as usize] = palette.len() as u8;
            palette.push(*color);
        }

        // Re-assign the palette indices to the data
        for i in 0..data.len() {
            data[i] = color_idx_lookup[data[i] as usize];
        }

        Self {
            height: bytes.len() as u32,
            width: bytes[0].len() as u32,
            palette,
            data,
        }
    }
}
