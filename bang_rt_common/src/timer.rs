use std::{
    thread,
    time::{Duration, Instant},
};

pub struct StdTime;

trait Time {
    fn now(&self) -> Instant;
    fn sleep(&self, duration: Duration);
}

impl Time for StdTime {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) {
        thread::sleep(duration);
    }
}

pub struct Timer<T> {
    time: T,
    period: Duration,
    margin: Duration,
    early_threshold: Duration,
    next: Instant,
    recent_timings: [Duration; 3],
}

impl Timer<StdTime> {
    pub fn new(target_fps: u64) -> Self {
        Self::with_gt(target_fps, StdTime)
    }
}

#[allow(private_bounds)]
impl<T: Time> Timer<T> {
    fn with_gt(target_fps: u64, time: T) -> Self {
        let period = Duration::from_secs_f64(1.0 / target_fps as f64);
        let next = time.now();
        Self {
            time,
            period,
            margin: Duration::from_micros(3500),
            early_threshold: Duration::from_micros(200),
            next,
            recent_timings: [
                Duration::default(),
                Duration::default(),
                Duration::default(),
            ],
        }
    }

    fn adjust_margin(&mut self, instant_after_sleep: Instant) {
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
        let instant_before_sleep = self.time.now();
        let time_left = self.next - instant_before_sleep;
        if time_left >= self.margin {
            self.time.sleep(time_left - self.margin);
        }

        let mut now = self.time.now();
        self.adjust_margin(now);

        while now < self.next {
            let snooze = (self.next - now) * 2 / 3;
            self.time.sleep(snooze);
            now = self.time.now();
        }
        self.next += self.period;
        self.next
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn test_timer() {
        fn s(i: u64) -> Duration {
            Duration::from_secs(i)
        }

        fn us(i: u64) -> Duration {
            Duration::from_micros(i)
        }

        struct TestTime<'a>(&'a Cell<&'a [Instant]>, &'a Cell<&'a [Duration]>);

        impl Time for TestTime<'_> {
            fn now(&self) -> Instant {
                let slice = self.0.get();
                self.0.set(&slice[1..]);
                slice[0]
            }
            fn sleep(&self, duration: Duration) {
                let slice = self.1.get();
                self.1.set(&slice[1..]);
                assert_eq!(duration, slice[0]);
            }
        }

        // Init

        let start = Instant::now();

        let now_init = [start];
        let now = Cell::<&[Instant]>::new(&now_init);
        let expected_sleep = Cell::<&[Duration]>::new(&[]);

        let mut timer = Timer::with_gt(1, TestTime(&now, &expected_sleep));

        // Frame 0, no sleep because next is now

        let now_0 = [start, start];
        let sleep_0 = [];
        now.set(&now_0);
        expected_sleep.set(&sleep_0);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(1));

        // Frame 1, regular sleep and overlong snooze that matches deadline perfectly

        let now_1 = [
            start,                   // At start
            start + s(1) - us(3500), // After sleep
            start + s(1),            // After first snooze
        ];
        let sleep_1 = [
            s(1) - us(3500),  // Sleep
            us(3500) * 2 / 3, // First snooze
        ];
        now.set(&now_1);
        expected_sleep.set(&sleep_1);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(2));

        // Frame 2, regular sleep and standard, iterative snooze

        let now_2 = [
            start + s(1),            // At start
            start + s(2) - us(3000), // After sleep
            start + s(2) - us(300),  // After first snooze
            start + s(2) - us(30),   // After second snooze
            start + s(2) - us(3),    // After third snooze
            start + s(3),            // After final
        ];
        let sleep_2 = [
            s(1) - us(3500), // Sleep
            us(2000),        // First snooze
            us(200),         // Second snooze
            us(20),          // Third snooze
            us(2),           // Final snooze
        ];
        now.set(&now_2);
        expected_sleep.set(&sleep_2);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(3));

        // Frame 3

        let now_3 = [
            start + s(2),            // At start
            start + s(3) - us(3000), // After sleep
            start + s(3),            // After snooze
        ];
        let sleep_3 = [
            s(1) - us(3500), // Sleep
            us(2000),        // Snooze
        ];
        now.set(&now_3);
        expected_sleep.set(&sleep_3);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(4));

        // Frame 4, the margin is now adjusted down from 3500 to 3125

        let now_4 = [
            start + s(3),           // At start
            start + s(4) - us(208), // After sleep, we land only a bit before the early threshold
            start + s(4),           // After snooze
        ];
        let sleep_4 = [
            s(1) - us(3125), // Sleep
            us(208) * 2 / 3, // Snooze
        ];
        now.set(&now_4);
        expected_sleep.set(&sleep_4);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(5));

        // Frame 5, the margin is still adjusted down from 3125 to 3125 - 208/8 = 3099

        let now_5 = [
            start + s(4),           // At start
            start + s(5) - us(199), // After sleep, we land to the wrong side of early threshold
            start + s(5),           // After snooze
        ];
        let sleep_5 = [
            s(1) - us(3099), // Sleep
            us(199) * 2 / 3, // Snooze
        ];
        now.set(&now_5);
        expected_sleep.set(&sleep_5);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(6));

        // Frame 6, the margin is no more adjusted because the early threshold is reached

        let now_6 = [
            start + s(5),           // At start
            start + s(6) - us(200), // After sleep
            start + s(6),           // After snooze
        ];
        let sleep_6 = [
            s(1) - us(3099), // Sleep
            us(200) * 2 / 3, // Snooze
        ];
        now.set(&now_6);
        expected_sleep.set(&sleep_6);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(7));

        // Frame 7, we sleep too much and are late, so the margin is adjusted backwards by 1000

        let now_7 = [
            start + s(6),            // At start
            start + s(7) + us(1000), // After sleep
        ];
        let sleep_7 = [
            s(1) - us(3099), // Sleep
        ];
        now.set(&now_7);
        expected_sleep.set(&sleep_7);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(8));

        // Frame 8, sleep according to the adjusted margin: 4099

        let now_8 = [
            start + s(7), // At start
            start + s(8), // After sleep
        ];
        let sleep_8 = [
            s(1) - us(4099), // Sleep
        ];
        now.set(&now_8);
        expected_sleep.set(&sleep_8);

        let next_deadline = timer.wait_until_next();
        assert_eq!(next_deadline, start + s(9));
    }
}
