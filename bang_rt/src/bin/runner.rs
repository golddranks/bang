use bang_rt::start_runtime;

fn main() {
    let mut args = std::env::args();
    let Some(libname) = args.nth(1) else {
        eprintln!("Usage: runner <libname>");
        std::process::exit(1);
    };
    eprintln!("Running {libname}");
    start_runtime(&libname);
}
