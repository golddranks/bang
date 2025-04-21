use std::cmp::{max, min};
use std::io::Write;

unsafe extern "C" {
    safe fn mach_absolute_time() -> u64;
    safe fn mach_timebase_info(info: *mut TimebaseInfo) -> i32;
    safe fn mach_wait_until(deadline: u64) -> i32;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct TimebaseInfo {
    numer: u32,
    denom: u32,
}

impl TimebaseInfo {
    fn new() -> Self {
        let mut info = TimebaseInfo { numer: 0, denom: 0 };
        mach_timebase_info(&mut info as *mut _);
        info
    }

    fn micros_to_abs(&self, micros: u64) -> u64 {
        (1000 * micros as u128 * self.denom as u128 / self.numer as u128) as u64
    }

    fn abs_to_micros(&self, abs: i64) -> i64 {
        (abs as i128 * self.numer as i128 / self.denom as i128) as i64 / 1000
    }
}

fn run_loop(hz: u32) {
    let info = TimebaseInfo::new();
    let period = info.micros_to_abs(1_000_000 / hz as u64);
    let too_early = info.micros_to_abs(200); // threshold for adjusting the margin & spinning
    let jiffy = info.micros_to_abs(10);

    let mut margin: u64 = info.micros_to_abs(3_500); // start with a big buffer
    let mut early: i64 = 0;
    let mut early_prev: i64 = 0;
    let mut early_prev_prev: i64;
    let mut next = mach_absolute_time() + period;

    println!(
        "Margin (μs)\tAfter wait; until deadline (μs)\tMargin adjusted (μs)\tWaited more; until deadline - margin = wait amount → actually waited (μs)\tAfter waits; until deadline (μs)\tSpinning jumps/interrupts?\tAfter spinning; until deadline (μs)"
    );

    let mut stdout = std::io::stdout().lock();

    loop {
        next += period;
        let wait_target = next - margin;
        write!(stdout, "{}\t", info.abs_to_micros(margin as i64)).unwrap();

        // main wait
        mach_wait_until(wait_target);
        let mut now = mach_absolute_time();

        early_prev_prev = early_prev;
        early_prev = early;
        early = next as i64 - now as i64;
        write!(stdout, "{}\t", info.abs_to_micros(early)).unwrap();

        // margin adjustment
        let adjust = if early < 0 {
            // Late: increase the margin by the late amount
            -early
        } else {
            let least_early = min(early, min(early_prev, early_prev_prev));
            if least_early > too_early as i64 {
                // Early: decrease the margin gently
                -least_early / 8
            } else {
                0
            }
        };
        margin = max(margin as i64 + adjust, 0) as u64;
        write!(stdout, "{:+}\t", info.abs_to_micros(adjust)).unwrap();

        // re-sleep if it's too early
        while now < next - too_early {
            let margin = (next - now) / 3; // sleeping only 2/3 of the remaining time seems to protect from overshooting
            let rewait_target = next - margin;
            write!(
                stdout,
                "{} - {} = {}",
                info.abs_to_micros(next as i64 - now as i64),
                info.abs_to_micros(margin as i64),
                info.abs_to_micros(next as i64 - now as i64 - margin as i64)
            )
            .unwrap();
            mach_wait_until(rewait_target);
            let after_wait = mach_absolute_time();
            write!(
                stdout,
                " → {}; ",
                info.abs_to_micros(after_wait as i64 - now as i64)
            )
            .unwrap();
            now = after_wait;
        }
        write!(
            stdout,
            "\t{}\t",
            info.abs_to_micros(next as i64 - now as i64)
        )
        .unwrap();

        // Spin until the precise deadline
        if now < next {
            let mut prev = now;
            loop {
                now = mach_absolute_time();
                if prev < now - jiffy {
                    write!(stdout, "{} ", info.abs_to_micros(next as i64 - prev as i64)).unwrap();
                    // Thread got interrupted at this point?
                    write!(stdout, "{} ", info.abs_to_micros(next as i64 - now as i64)).unwrap();
                }
                prev = now;
                if now >= next {
                    break;
                }
            }
        }
        write!(stdout, "\t").unwrap();

        // Loop start
        now = mach_absolute_time();
        writeln!(stdout, "{}", info.abs_to_micros(next as i64 - now as i64)).unwrap();

        // Doing work ...
    }
}

fn main() {
    run_loop(120);
}
