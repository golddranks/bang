use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

unsafe extern "C" {
    unsafe fn signal(sig: i32, handler: extern "C" fn(i32)) -> extern "C" fn(i32);
}

#[derive(Debug)]
pub struct Ender {
    should_end: AtomicBool,
    notify_end: fn(ender: &Ender),
}

static SHOULD_END: AtomicPtr<AtomicBool> = AtomicPtr::new(null_mut());

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

    extern "C" fn sigint_handler(_sig: i32) {
        let should_end = unsafe { &*SHOULD_END.load(Ordering::Acquire) };
        should_end.store(true, Ordering::Release);
    }

    #[cfg(target_os = "macos")]
    pub fn install_global_signal_handler(&self) {
        pub const SIGINT: i32 = 2;

        SHOULD_END.store(
            &self.should_end as *const AtomicBool as *mut AtomicBool,
            Ordering::Release,
        );

        unsafe {
            signal(SIGINT, Ender::sigint_handler);
        }
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

    #[test]
    fn test_signal_handler() {
        let ender = Ender::new(|_| CALLED.store(true, Ordering::Release));
        static CALLED: AtomicBool = AtomicBool::new(false);
        assert!(ender.should_end().not());
        ender.install_global_signal_handler();

        Ender::sigint_handler(2);

        assert!(ender.should_end());
        assert!(CALLED.load(Ordering::Acquire).not());
    }
}
