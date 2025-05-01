use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct Ender {
    should_end: AtomicBool,
    notify_end: fn(ender: &Ender),
}

impl Ender {
    pub fn new(notify_end: fn(ender: &Ender)) -> Self {
        Ender {
            should_end: AtomicBool::new(false),
            notify_end,
        }
    }

    pub fn should_end(&self) -> bool {
        self.should_end.load(Ordering::Acquire)
    }

    pub fn soft_quit(&self) {
        self.should_end.store(true, Ordering::Release);
        (self.notify_end)(self);
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Not;

    use super::*;

    #[test]
    fn test_end() {
        let ender = Ender::new(|_| CALLED.store(true, Ordering::Release));
        static CALLED: AtomicBool = AtomicBool::new(false);
        assert!(ender.should_end().not());
        ender.soft_quit();
        assert!(ender.should_end());
        assert!(CALLED.load(Ordering::Acquire));
    }
}
