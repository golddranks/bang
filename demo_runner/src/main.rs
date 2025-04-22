use bang_rt::start_runtime_with_static;
use demo_impl::frame_logic_no_mangle;

fn main() {
    eprintln!("Running statically");
    start_runtime_with_static(frame_logic_no_mangle);
}
