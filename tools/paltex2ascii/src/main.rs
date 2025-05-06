use std::{
    env::args,
    fs::read,
    io::{Read, Write, stdin, stdout},
};

use bang_rt_common::{die, error::OrDie};

const CSI: &str = "\x1b[";

fn color_block(mut out: impl Write, ansi_color: u8) {
    write!(out, "{CSI}{}m██", 30 + ansi_color % 8).expect("Failed to write to stdout");
}

fn main() {
    let input = if let Some(path) = args().nth(1) {
        read(&path).or_(die!("Failed to read from file {:?}", path))
    } else {
        let mut input = Vec::new();
        stdin()
            .lock()
            .read_to_end(&mut input)
            .expect("Failed to read from stdin.");
        input
    };
    let paltex = paltex::decode(&input);

    let mut stdout = stdout().lock();

    writeln!(
        stdout,
        "Palette: (length: {} RGBA colors + 1 implicit transparent)",
        paltex.palette.len() - 1
    )
    .expect("Failed to write to stdout");

    for (i, color) in paltex.palette.iter().enumerate() {
        color_block(&mut stdout, i as u8);
        writeln!(stdout, "{CSI}0m Color {:?}", color.to_rgba_u8())
            .expect("Failed to write to stdout");
    }

    let mut row = Vec::new();
    for chunk in paltex.data.chunks(paltex.width as usize) {
        row.clear();
        for &color_idx in chunk {
            color_block(&mut row, color_idx);
        }
        row.push(b'\n');
        stdout.write_all(&row).expect("Failed to write to stdout");
    }
}
