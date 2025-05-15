use bang_rt_common::runtime::start_static;
use bang_rt_macos::MacOSRT;
use demo_main::DemoLogic;

fn main() {
    eprintln!("Running statically");
    start_static(MacOSRT, DemoLogic);
}
