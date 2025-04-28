use std::{
    thread::sleep,
    time::{Duration, Instant},
};

pub struct Timer {
    period: Duration,
    margin: Duration,
    early_threshold: Duration,
    next: Instant,
    recent_timings: [Duration; 3],
}

impl Timer {
    pub fn new(target_fps: u64) -> Self {
        let period = Duration::from_secs_f64(1.0 / target_fps as f64);
        Self {
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

    pub fn wait_until_next(&mut self) -> Instant {
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
        self.next += self.period;
        self.next
    }
}
