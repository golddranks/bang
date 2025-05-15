use std::thread::{self, sleep};
use std::time::Duration;
use std::{io, ops::Not, os::unix::io::AsRawFd};

use bang_core::Config;
use bang_core::ffi::RtCtx;
use bang_rt_common::die;
use bang_rt_common::end::Ender;
use bang_rt_common::error::OrDie;
use bang_rt_common::{draw::DrawReceiver, input::InputGatherer};

use crate::draw::{draw, flush, show_cursor};
use crate::{LOOP_MS, input};

unsafe extern "C" {
    safe fn tcgetattr(fd: i32, termios_p: &mut Termios) -> i32;
    safe fn tcsetattr(fd: i32, optional_actions: i32, termios_p: &Termios) -> i32;
    unsafe fn fcntl(fd: i32, cmd: i32, ...) -> i32;
}

// TODO: untested!
#[cfg(target_os = "linux")]
#[allow(nonstandard_style)]
mod platform {
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
mod platform {
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

    pub const F_GETFL: i32 = 3;
    pub const F_SETFL: i32 = 4;
    pub const O_NONBLOCK: i32 = 0x0004;
}

use platform::*;

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

        unsafe {
            let flags = fcntl(fd, F_GETFL);
            fcntl(fd, F_SETFL, flags | O_NONBLOCK);
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
        _rt_ctx: &mut RtCtx,
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &'l Ender,
        _config: &'l Config,
    ) -> Self {
        let _terminal_mode = TerminalMode::new().or_(die!("Failed to initialize terminal mode"));
        Window {
            input_gatherer,
            draw_receiver,
            ender,
            _terminal_mode,
        }
    }

    pub fn run(&mut self) {
        let gatherer = &mut self.input_gatherer;
        let draw_receiver = &mut self.draw_receiver;
        thread::scope(|s| {
            s.spawn(|| input::gather(self.ender, gatherer));
            Self::render_loop(self.ender, draw_receiver);
        });
    }

    fn render_loop(ender: &'l Ender, draw_receiver: &mut DrawReceiver<'l>) {
        let mut buf = Vec::new();
        let mut output_stream = std::io::stdout().lock();

        while ender.should_end().not() {
            if draw_receiver.has_fresh() {
                let frame = draw_receiver.get_fresh();
                draw(frame, &mut output_stream, &mut buf);
            }
            sleep(Duration::from_millis(LOOP_MS));
        }
        show_cursor(&mut buf);
        flush(&mut buf, &mut output_stream);
    }
}
