use bang_rt::{as_frame_logic, start_runtime_with_static};
use demo_impl::frame_logic;

fn main() {
    eprintln!("Running statically");
    let fl = as_frame_logic(frame_logic);
    start_runtime_with_static(fl);
}
