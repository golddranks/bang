use std::{
    ffi::{CStr, c_char, c_double, c_longlong, c_schar, c_ulonglong, c_void},
    mem::transmute,
    ops::Not,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

pub(super) type Ptr = *mut c_void;
pub(super) type Imp = unsafe extern "C" fn() -> *const c_void;
pub(super) type CStrPtr = *const c_char;
pub(super) type NSInteger = c_longlong;
pub(super) type NSUInteger = c_ulonglong;
pub(super) type Bool = c_schar;
pub(super) type CGFloat = c_double;

unsafe extern "C" {
    unsafe fn objc_getClass(name: CStrPtr) -> Ptr;
    unsafe fn sel_registerName(name: CStrPtr) -> Ptr;
    unsafe fn objc_msgSend();
    pub(super) unsafe fn class_addMethod(cls: Ptr, name: Ptr, imp: Imp, types: CStrPtr) -> Bool;
}

pub(super) unsafe fn to_imp2<A0, A1, R>(f: fn(A0, A1) -> R) -> Imp {
    unsafe { transmute(f) }
}

#[repr(transparent)]
pub(super) struct Obj(AtomicPtr<c_void>);

impl Obj {
    pub(super) const fn uninit() -> Self {
        Self(AtomicPtr::new(null_mut()))
    }

    pub(super) fn get(&self) -> Ptr {
        let ptr = self.0.load(Ordering::Relaxed);
        assert!(ptr.is_null().not());
        ptr
    }

    fn set(&self, ptr: Ptr) {
        assert!(ptr.is_null().not());
        self.0.store(ptr, Ordering::Relaxed);
    }
}

#[repr(transparent)]
pub(super) struct Cls(pub(super) Obj);

impl Cls {
    pub(super) const fn uninit() -> Self {
        Self(Obj::uninit())
    }

    pub(super) fn get(&self) -> Ptr {
        self.0.get()
    }

    pub(super) fn init(&self, name: &CStr) {
        self.0.set(unsafe { objc_getClass(name.as_ptr()) });
    }
}

#[repr(transparent)]
pub(super) struct Sel(Obj);

impl Sel {
    pub(super) const fn uninit() -> Self {
        Self(Obj::uninit())
    }

    pub(super) fn get(&self) -> Ptr {
        self.0.get()
    }

    pub(super) fn init(&self, name: &CStr) {
        self.0.set(unsafe { sel_registerName(name.as_ptr()) });
    }
}

pub(super) unsafe fn msg0<R>(receiver: &Obj, selector: &Sel) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: Ptr, selector: Ptr) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver.get(), selector.get())
    }
}

pub(super) unsafe fn msg1<A0, R>(receiver: &Obj, selector: &Sel, arg0: A0) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: Ptr, selector: Ptr, arg0: A0) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver.get(), selector.get(), arg0)
    }
}

pub(super) unsafe fn msg2<A0, A1, R>(receiver: &Obj, selector: &Sel, arg0: A0, arg1: A1) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: Ptr, selector: Ptr, arg0: A0, arg1: A1) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver.get(), selector.get(), arg0, arg1)
    }
}

pub(super) unsafe fn msg4<A0, A1, A2, A3, R>(
    receiver: &Obj,
    selector: &Sel,
    arg0: A0,
    arg1: A1,
    arg2: A2,
    arg3: A3,
) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(
                receiver: Ptr,
                selector: Ptr,
                arg0: A0,
                arg1: A1,
                arg2: A2,
                arg3: A3,
            ) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver.get(), selector.get(), arg0, arg1, arg2, arg3)
    }
}
