use std::ops::Not;

use bang_core::draw::AsBytes;

use crate::common::{Header, PalTex};

const RUN_MAX: usize = 0b0000_0111 + 1;

pub(crate) fn encode_op(color_idx: u8, run_dir: u8, run_len: u8) -> u8 {
    let mut op = u8::min(color_idx, 15) & 0b0000_1111;
    op |= u8::min(run_dir, 1) << 7;
    op |= (u8::min(run_len, RUN_MAX as u8).saturating_sub(1) & 0b0000_0111) << 4;
    op
}

pub(crate) fn encode_headers(paltex: &PalTex, output: &mut Vec<u8>) {
    let header = Header {
        width: paltex.width as u16,
        height: paltex.height as u16,
        pal_len: paltex.palette.len() as u16,
    };
    output.extend_from_slice(header.as_bytes());
    output.extend_from_slice(paltex.palette.as_bytes());
}

pub(crate) fn test_run_len(
    data: &[u8],
    drained: &[bool],
    stride: usize,
    mut target: usize,
    color_idx: u8,
) -> usize {
    let mut run_len = 1;
    while run_len < RUN_MAX {
        target += stride;
        if target >= data.len() || data[target] != color_idx {
            break;
        }
        if drained[target].not() {
            run_len += 1;
        }
    }
    run_len
}

pub(crate) fn encode_main(data: &[u8], width: usize, output: &mut Vec<u8>) {
    let mut cursor = 0;
    let mut drained = vec![false; data.len()];

    while cursor < data.len() {
        let color_idx = data[cursor];

        let h_run_len = test_run_len(data, &drained, 1, cursor, color_idx);
        let v_run_len = test_run_len(data, &drained, width, cursor, color_idx);

        let (run_len, run_dir) = if h_run_len > v_run_len {
            (h_run_len, 0)
        } else {
            (v_run_len, 1)
        };

        let stride = if run_dir == 0 { 1 } else { width };
        let mut drain_len = 0;
        let mut target = cursor;
        while drain_len < run_len {
            if drained[target].not() {
                drained[target] = true;
                drain_len += 1;
            }
            target += stride;
        }

        output.push(encode_op(color_idx, run_dir, run_len as u8));

        while cursor < data.len() && drained[cursor] {
            cursor += 1;
        }
    }
}

pub fn encode(paltex: &PalTex, output: &mut Vec<u8>) {
    encode_headers(paltex, output);
    encode_main(&paltex.data, paltex.width as usize, output);
}
