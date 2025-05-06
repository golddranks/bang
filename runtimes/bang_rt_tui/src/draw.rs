use std::io::{StdoutLock, Write};

use bang_core::draw::{Cmd, DrawFrame};
use bang_rt_common::{die, error::OrDie};

const CSI: &str = "\x1b[";

fn move_to(buf: &mut Vec<u8>, row: u32, col: u32) {
    write!(buf, "{CSI}{row};{col}H").or_(die!("Error moving cursor"));
}

fn erase_screen(buf: &mut Vec<u8>) {
    write!(buf, "{CSI}2J").or_(die!("Error erasing screen"));
}

fn hide_cursor(buf: &mut Vec<u8>) {
    write!(buf, "{CSI}?25l").or_(die!("Error hiding cursor"));
}

pub fn show_cursor(buf: &mut Vec<u8>) {
    write!(buf, "{CSI}?25h").or_(die!("Error showing cursor"));
}

pub fn flush(buf: &mut Vec<u8>, output_stream: &mut StdoutLock<'static>) {
    move_to(buf, 0, 0);
    output_stream
        .write_all(buf)
        .or_(die!("Error writing to stdout"));
    output_stream.flush().or_(die!("Error flushing stdout"));
}

pub fn draw(frame: &DrawFrame, output_stream: &mut StdoutLock<'static>, buf: &mut Vec<u8>) {
    buf.clear();
    erase_screen(buf);
    hide_cursor(buf);
    let chars = "████";
    for cmd in frame.cmds {
        match cmd {
            Cmd::DrawSQuads { pos, .. } => {
                for pos in pos.iter() {
                    let row = ((pos.y + 200.0) / 20.0) as u32;
                    let col = ((pos.x + 200.0) / 10.0) as u32;
                    for r in 0..3 {
                        move_to(buf, row + r, col);
                        write!(buf, "{chars}").or_(die!("Error writing to buffer"));
                    }
                }
                flush(buf, output_stream);
            }
        }
    }
}
