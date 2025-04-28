use std::time::{Duration, Instant};

use crate::objc::wrappers::{NSProcessInfo, NSTimeInterval};

#[derive(Debug)]
pub struct TimeConverter {
    start_instant: Instant,
    start_sys: NSTimeInterval,
}

impl TimeConverter {
    pub fn new() -> Self {
        let process_info = NSProcessInfo::IPtr::process_info();
        Self {
            start_instant: Instant::now(),
            start_sys: process_info.system_uptime(),
        }
    }

    pub fn sys_to_instant(&self, sys_time: NSTimeInterval) -> Instant {
        self.start_instant + Duration::from_secs_f64((sys_time - self.start_sys).to_secs())
    }
}
