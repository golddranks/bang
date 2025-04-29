use std::{
    mem::transmute,
    ops::Not,
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

static NOTIFY_END: AtomicPtr<()> = AtomicPtr::new(null_mut());
static END: AtomicBool = AtomicBool::new(false);

pub fn should_end() -> bool {
    END.load(Ordering::Acquire)
}

pub fn soft_quit() {
    END.store(true, Ordering::Release);
    let ptr = NOTIFY_END.load(Ordering::Acquire);
    if ptr.is_null().not() {
        let notify_end = unsafe { transmute::<*mut (), fn()>(ptr) };
        notify_end();
    }
}

pub fn init_notify_end(notify_end: fn()) {
    NOTIFY_END.store(notify_end as *mut (), Ordering::Release);
}
