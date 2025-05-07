use bang_core::draw::AsBytes;

use crate::common::{Color, Header, InvalidInput, PalTex};

pub(crate) fn decode_header(input: &[u8]) -> Result<Header, InvalidInput> {
    let mut header = Header::default();
    let header_end = size_of::<Header>();

    if input.len() < header_end {
        return Err(InvalidInput);
    }

    header.as_bytes_mut().copy_from_slice(&input[..header_end]);

    header.pal_len = u8::min(header.pal_len, 15);
    if header.width == 0 || header.height == 0 {
        return Err(InvalidInput);
    }

    Ok(header)
}

pub(crate) fn decode_palette<'a>(
    header: &Header,
    input: &'a [u8],
) -> Result<(Vec<Color>, &'a [u8]), InvalidInput> {
    let mut palette_vec = vec![Color::TRANSPARENT; header.pal_len as usize + 1];
    let palette = &mut palette_vec[1..];
    let header_end = size_of::<Header>();
    let palette_end = header_end + size_of_val(palette);
    if input.len() < palette_end {
        return Err(InvalidInput);
    }
    palette
        .as_bytes_mut()
        .copy_from_slice(&input[header_end..palette_end]);
    let img_data = &input[palette_end..];

    Ok((palette_vec, img_data))
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
    while filled < run_len && cursor + i * stride < data.len() {
        if data[cursor + i * stride] == u8::MAX {
            data[cursor + i * stride] = color_idx;
            filled += 1;
        }
        i += 1;
    }
}

pub(crate) fn decode_main(header: &Header, encoded_data: &[u8]) -> Vec<u8> {
    let max_len = encoded_data.len() * 8;
    let len = usize::min(header.width as usize * header.height as usize, max_len);
    let mut data = vec![u8::MAX; len];

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

    data
}

pub fn decode(input: &[u8]) -> Result<PalTex, InvalidInput> {
    let header = decode_header(input)?;
    let (palette, encoded_data) = decode_palette(&header, input)?;
    let data = decode_main(&header, encoded_data);

    Ok(PalTex {
        width: header.width as u32,
        height: header.height as u32,
        palette,
        data,
    })
}
