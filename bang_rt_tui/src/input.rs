use std::{
    io::{ErrorKind, Read},
    ops::Not,
    thread,
    time::{Duration, Instant},
};

use bang_core::input::{Key, KeyState};
use bang_rt_common::{end::Ender, input::InputGatherer};

use crate::LOOP_MS;

pub fn gather(ender: &Ender, input_gatherer: &mut InputGatherer) {
    let mut input_stream = std::io::stdin().lock();
    let mut input_buf = [0u8; 1];
    while ender.should_end().not() {
        match input_stream.read(&mut input_buf) {
            Ok(1) => {}
            Ok(_) => unreachable!(),
            Err(e) => {
                if let ErrorKind::WouldBlock | ErrorKind::Interrupted = e.kind() {
                    thread::sleep(Duration::from_millis(LOOP_MS));
                    continue;
                } else {
                    panic!("Error reading input: {}", e);
                }
            }
        }
        let key = Key::from_ascii(input_buf[0]);
        input_gatherer.update(key, KeyState::Tap, Instant::now());
    }
}
