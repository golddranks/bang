use std::io::Read;
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use std::{io, ops::Not, os::unix::io::AsRawFd};

use bang_core::input::{Key, KeyState};
use bang_rt_common::end::Ender;
use bang_rt_common::error::OrDie;
use bang_rt_common::{draw::DrawReceiver, input::InputGatherer};

use crate::draw::draw;

unsafe extern "C" {
    pub safe fn tcgetattr(fd: i32, termios_p: &mut Termios) -> i32;
    pub safe fn tcsetattr(fd: i32, optional_actions: i32, termios_p: &Termios) -> i32;
}

#[cfg(target_os = "linux")]
#[allow(nonstandard_style)]
mod termios {
    type cc_t = u8;
    type speed_t = u32;
    type tcflag_t = u32;

    pub const ICANON: tcflag_t = 0o000002;
    pub const ECHO: tcflag_t = 0o000010;
    pub const TCSANOW: i32 = 0;
    pub const VMIN: usize = 6;
    pub const VTIME: usize = 5;

    #[repr(C)]
    #[derive(Clone, Copy, Default, Debug)]
    pub struct Termios {
        pub c_iflag: tcflag_t,
        pub c_oflag: tcflag_t,
        pub c_cflag: tcflag_t,
        pub c_lflag: tcflag_t,
        pub c_line: cc_t,
        pub c_cc: [cc_t; 32],
        pub c_ispeed: speed_t,
        pub c_ospeed: speed_t,
    }
}

#[cfg(target_os = "macos")]
#[allow(nonstandard_style)]
mod termios {
    type cc_t = u8;
    type speed_t = u64;
    type tcflag_t = u64;

    pub const ICANON: tcflag_t = 0x00000100;
    pub const ECHO: tcflag_t = 0x00000008;
    pub const TCSANOW: i32 = 0;
    pub const VMIN: usize = 16;
    pub const VTIME: usize = 17;

    #[repr(C)]
    #[derive(Clone, Copy, Default, Debug)]
    pub struct Termios {
        pub c_iflag: tcflag_t,
        pub c_oflag: tcflag_t,
        pub c_cflag: tcflag_t,
        pub c_lflag: tcflag_t,
        pub c_cc: [cc_t; 20],
        pub c_ispeed: speed_t,
        pub c_ospeed: speed_t,
    }
}

use termios::*;

pub struct TerminalMode {
    original: Termios,
    fd: i32,
}

impl TerminalMode {
    pub fn new() -> io::Result<Self> {
        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();
        let mut termios = Termios::default();
        if tcgetattr(fd, &mut termios) != 0 {
            return Err(io::Error::last_os_error());
        }
        let original = termios;

        // Disable canonical mode and echo
        termios.c_lflag &= !(ICANON | ECHO);
        termios.c_cc[VMIN] = 1;
        termios.c_cc[VTIME] = 0;

        if tcsetattr(fd, TCSANOW, &termios) != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(TerminalMode { original, fd })
    }
}

impl Drop for TerminalMode {
    fn drop(&mut self) {
        tcsetattr(self.fd, TCSANOW, &self.original);
    }
}
pub struct Window<'l> {
    input_gatherer: InputGatherer<'l>,
    draw_receiver: DrawReceiver<'l>,
    ender: &'l Ender,
    _terminal_mode: TerminalMode,
}

impl<'l> Window<'l> {
    pub fn init(
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &'l Ender,
    ) -> Self {
        let _terminal_mode = TerminalMode::new().or_die("Failed to initialize terminal mode");
        Window {
            input_gatherer,
            draw_receiver,
            ender,
            _terminal_mode,
        }
    }

    pub fn run(&mut self) {
        thread::scope(|s| {
            s.spawn(|| {
                let mut input_stream = std::io::stdin().lock().bytes();
                while self.ender.should_end().not() {
                    for byte in &mut input_stream {
                        let byte = byte.or_die("Error reading byte");
                        let state = KeyState::Tap;
                        let key = Key::from_ascii(byte);
                        self.input_gatherer.update(key, state, Instant::now());
                    }
                }
            });
            let mut buf = Vec::new();
            let mut output_stream = std::io::stdout().lock();
            while self.ender.should_end().not() {
                if self.draw_receiver.has_fresh() {
                    let frame = self.draw_receiver.get_fresh();
                    draw(frame, &mut output_stream, &mut buf);
                }
                sleep(Duration::from_millis(10));
            }
        });
    }
}
