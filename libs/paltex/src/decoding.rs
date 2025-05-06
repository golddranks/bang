use bang_core::draw::AsBytes;

use crate::common::{Color, Header, PalTex};

pub(crate) fn decode_header(input: &[u8]) -> Header {
    let mut header = Header::default();
    let header_end = size_of::<Header>();
    header.as_bytes_mut().copy_from_slice(&input[..header_end]);

    header.pal_len = u8::min(header.pal_len, 15);

    header
}

pub(crate) fn decode_palette<'a>(header: &Header, input: &'a [u8]) -> (Vec<Color>, &'a [u8]) {
    let mut palette_vec = vec![Color::TRANSPARENT; header.pal_len as usize + 1];
    let palette = &mut palette_vec[1..];
    let header_end = size_of::<Header>();
    let palette_end = header_end + size_of_val(palette);
    palette
        .as_bytes_mut()
        .copy_from_slice(&input[header_end..palette_end]);
    let img_data = &input[palette_end..];

    (palette_vec, img_data)
}

pub(crate) fn decode_op(op: u8) -> (u8, u8, u8) {
    let color_idx = op & 0b0000_1111;
    let run_dir = op >> 7;
    let run_len = ((op >> 4) & 0b0000_0111) + 1;

    (color_idx, run_dir, run_len)
}

pub(crate) fn perform_op(
    data: &mut [u8],
    cursor: usize,
    color_idx: u8,
    stride: usize,
    run_len: u8,
) {
    let mut i = 0;
    let mut filled = 0;
    while filled < run_len {
        if data[cursor + i * stride] == u8::MAX {
            data[cursor + i * stride] = color_idx;
            filled += 1;
        }
        i += 1;
    }
}

pub(crate) fn decode_main(header: &Header, encoded_data: &[u8]) -> Vec<u8> {
    let mut data = vec![u8::MAX; header.width as usize * header.height as usize];

    let mut cursor = 0;
    for &op in encoded_data {
        let (color_idx, run_dir, run_len) = decode_op(op);

        let stride = if run_dir == 0 {
            1
        } else {
            header.width as usize
        };

        perform_op(&mut data, cursor, color_idx, stride, run_len);

        while cursor < data.len() && data[cursor] != u8::MAX {
            cursor += 1;
        }
    }

    data.truncate(cursor.next_multiple_of(header.width as usize));
    for color_idx in &mut data[cursor..] {
        *color_idx = 0;
    }

    data
}

pub fn decode(input: &[u8]) -> PalTex {
    let header = decode_header(input);
    let (palette, encoded_data) = decode_palette(&header, input);
    let data = decode_main(&header, encoded_data);

    PalTex {
        width: header.width as u32,
        height: header.height as u32,
        palette,
        data,
    }
}
