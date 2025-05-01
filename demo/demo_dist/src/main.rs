use bang_rt_common::{load::InlinedFrameLogic, runtime::start_static};
use bang_rt_macos::MacOSRT;
use demo_impl::frame_logic;

fn main() {
    eprintln!("Running statically");
    let fl = InlinedFrameLogic::new(frame_logic);

    start_static(MacOSRT, fl);
}
