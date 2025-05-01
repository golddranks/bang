use std::ffi::CString;

use bang_rt_common::{error::OrDie, runtime::start_dynamic};
use bang_rt_macos::MacOSRT;

fn main() {
    let mut args = std::env::args();
    let Some(libname) = args.nth(1) else {
        eprintln!("Usage: runner <library path>");
        std::process::exit(1);
    };
    eprintln!("Running {libname} dynamically on MacOS");

    start_dynamic(MacOSRT, &CString::new(libname).or_die("Invalid path"));
}
