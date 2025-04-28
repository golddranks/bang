use bang_rt_common::start_rt_dynamic;
use bang_rt_tui::TuiRT;

fn main() {
    let mut args = std::env::args();
    let Some(libname) = args.nth(1) else {
        eprintln!("Usage: runner <libname>");
        std::process::exit(1);
    };
    eprintln!("Running {libname} dynamically in TUI mode");

    start_rt_dynamic::<TuiRT>(&libname);
}
