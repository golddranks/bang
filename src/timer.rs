use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::objc::wrappers::{NSProcessInfo, NSTimeInterval};

pub struct Timer {
    start_instant: Instant,
    start_sys: NSTimeInterval,
    period: Duration,
    margin: Duration,
    early_threshold: Duration,
    next: Instant,
    recent_timings: [Duration; 3],
}

static DEADLINE: AtomicU64 = AtomicU64::new(0);

impl Timer {
    pub fn deadline() -> NSTimeInterval {
        NSTimeInterval::from_u64(DEADLINE.load(Ordering::Acquire))
    }

    pub fn new(target_fps: u64) -> Self {
        let period = Duration::from_secs_f64(1.0 / target_fps as f64);
        let process_info = NSProcessInfo::IPtr::process_info();
        Self {
            start_instant: Instant::now(),
            start_sys: process_info.system_uptime(),
            period,
            margin: Duration::from_micros(3500),
            early_threshold: Duration::from_micros(200),
            next: Instant::now(),
            recent_timings: [
                Duration::default(),
                Duration::default(),
                Duration::default(),
            ],
        }
    }

    fn adjust_margin(&mut self) {
        let instant_after_sleep = Instant::now();

        let how_early = self.next - instant_after_sleep;
        self.recent_timings.rotate_right(1);
        self.recent_timings[0] = how_early;

        if instant_after_sleep > self.next {
            // Late: increase the margin by the full late amount
            self.margin += instant_after_sleep - self.next;
        } else {
            let least_early = *self.recent_timings.iter().min().expect("UNREACHABLE");
            if least_early > self.early_threshold {
                // A lot of earlies recently: decrease the margin gently
                self.margin -= least_early / 8;
            }
        }
    }

    fn set_deadline(&self) {
        let since_start = self.next - self.start_instant;
        let next_sys = self.start_sys + since_start.as_secs_f64();
        DEADLINE.store(next_sys.to_u64(), Ordering::Release);
    }

    pub fn wait_until_next(&mut self) {
        self.next += self.period;
        self.set_deadline();
        let instant_before_sleep = Instant::now();
        let time_left = self.next - instant_before_sleep;
        if time_left >= self.margin {
            sleep(time_left - self.margin);
        }

        self.adjust_margin();

        let mut now = Instant::now();
        while now < self.next {
            let snooze = (self.next - now) * 2 / 3;
            sleep(snooze);
            now = Instant::now();
        }
    }
}
