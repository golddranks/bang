use std::{
    thread::sleep,
    time::{Duration, Instant},
};

pub struct Timer {
    period: Duration,
    margin: Duration,
    early_threshold: Duration,
    next: Instant,
    recent_early: [Duration; 3],
}

impl Timer {
    pub fn new(target_fps: u64) -> Self {
        let period = Duration::from_secs_f64(1.0 / target_fps as f64);
        Self {
            period,
            margin: Duration::from_micros(3500),
            early_threshold: Duration::from_micros(200),
            next: Instant::now(),
            recent_early: [
                Duration::default(),
                Duration::default(),
                Duration::default(),
            ],
        }
    }

    fn adjust_margin(&mut self) {
        let instant_after_sleep = Instant::now();

        let early = self.next - instant_after_sleep;
        self.recent_early.rotate_right(1);
        self.recent_early[0] = early;

        if instant_after_sleep > self.next {
            // Late: increase the margin by the late amount
            self.margin += instant_after_sleep - self.next;
        } else {
            let least_early = *self.recent_early.iter().min().expect("UNREACHABLE");
            if least_early > self.early_threshold {
                // A lot of earlies recently: decrease the margin gently
                self.margin -= least_early / 8;
            }
        }
    }

    pub fn wait_until_next(&mut self) {
        self.next += self.period;
        let instant_before_sleep = Instant::now();
        sleep(self.next - instant_before_sleep - self.margin);

        self.adjust_margin();

        let mut now = Instant::now();
        while now < self.next - self.early_threshold {
            let snooze = (self.next - now) * 2 / 3;
            println!("snoozing for {}", snooze.as_micros());
            sleep(snooze);
            now = Instant::now();
        }

        println!("{}", (self.next - now).as_micros());
    }
}
