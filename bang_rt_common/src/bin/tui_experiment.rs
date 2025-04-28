use std::io::{self, Read};
use std::os::unix::io::AsRawFd;

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

fn main() {
    let _tm = TerminalMode::new().expect("fail");

    println!("Press 'q' to quit");
    #[allow(clippy::unbuffered_bytes)]
    for byte in std::io::stdin().bytes() {
        let b = byte.unwrap();
        if b == b'q' {
            break;
        }
        println!("You pressed: {:?}", b as char);
    }
}
