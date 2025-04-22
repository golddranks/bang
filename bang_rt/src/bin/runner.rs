use bang_rt::start_runtime_with_dynamic;

fn main() {
    let mut args = std::env::args();
    let Some(libname) = args.nth(1) else {
        eprintln!("Usage: runner <libname>");
        std::process::exit(1);
    };
    eprintln!("Running {libname} dynamically");
    start_runtime_with_dynamic(&libname);
}
