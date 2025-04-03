use std::{
    ffi::{CStr, c_char, c_double, c_schar, c_ulonglong, c_void},
    mem::transmute,
    ops::Not,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

pub(super) type Ptr = *mut c_void;
pub(super) type Imp = unsafe extern "C" fn() -> *const c_void;
pub(super) type CStrPtr = *const c_char;
pub(super) type NSUInteger = c_ulonglong;
pub(super) type Bool = c_schar;
pub(super) type CGFloat = c_double;

unsafe extern "C" {
    unsafe fn objc_allocateClassPair(class: Ptr, name: CStrPtr, extraBytes: usize) -> Ptr;
    unsafe fn objc_registerClassPair(class: Ptr);
    unsafe fn objc_getClass(name: CStrPtr) -> Ptr;
    unsafe fn sel_registerName(name: *const u8) -> Ptr;
    unsafe fn objc_msgSend();
    unsafe fn class_addMethod(cls: Ptr, name: Ptr, imp: Imp, types: CStrPtr) -> Bool;
}

pub(super) fn subclass(class: &Cls, name: &CStr) -> Cls {
    Cls(Obj::new(unsafe {
        objc_allocateClassPair(class.get(), name.as_ptr(), 0)
    }))
}

pub(super) fn register_class(class: &Cls) {
    unsafe { objc_registerClassPair(class.get()) }
}

unsafe fn to_imp2<R, A0, A1>(f: extern "C" fn(A0, A1) -> R) -> Imp {
    unsafe { transmute(f) }
}

unsafe fn to_imp3<R, A0, A1, A2>(f: extern "C" fn(A0, A1, A2) -> R) -> Imp {
    unsafe { transmute(f) }
}

pub(super) unsafe fn add_method2<R, A0, A1>(
    cls: &Cls,
    sel: &Sel,
    fn_ptr: extern "C" fn(A0, A1) -> R,
    types: &CStr,
) {
    unsafe {
        class_addMethod(cls.get(), sel.get(), to_imp2(fn_ptr), types.as_ptr());
    }
}

pub(super) unsafe fn add_method3<R, A0, A1, A2>(
    cls: &Cls,
    sel: &Sel,
    fn_ptr: extern "C" fn(A0, A1, A2) -> R,
    types: &CStr,
) {
    unsafe {
        class_addMethod(cls.get(), sel.get(), to_imp3(fn_ptr), types.as_ptr());
    }
}

#[derive(Debug)]
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

    pub(super) fn or_die(self, msg: &str) -> Self {
        if self.is_null() {
            eprintln!("Stuff is null: {msg}");
            panic!()
        }
        self
    }

    pub(super) fn is_null(&self) -> bool {
        self.0.load(Ordering::Relaxed).is_null()
    }

    pub(super) fn new(ptr: Ptr) -> Self {
        assert!(ptr.is_null().not());
        let obj = Obj::uninit();
        obj.set(ptr);
        obj
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

    pub(super) fn init_with(&self, cls: Cls) {
        self.0.set(cls.0.get());
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
        self.0
            .set(unsafe { sel_registerName(name.as_ptr() as *const u8) });
    }

    pub(super) fn init_from_underscored_literal(&self, name: &str) {
        let mut colon_name = name.replace("_", ":");
        colon_name.push('\0');
        self.0.set(unsafe { sel_registerName(colon_name.as_ptr()) });
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

pub(super) unsafe fn msg1<R, A0>(receiver: &Obj, selector: &Sel, arg0: A0) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: Ptr, selector: Ptr, arg0: A0) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver.get(), selector.get(), arg0)
    }
}

pub(super) unsafe fn msg2<R, A0, A1>(receiver: &Obj, selector: &Sel, arg0: A0, arg1: A1) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: Ptr, selector: Ptr, arg0: A0, arg1: A1) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver.get(), selector.get(), arg0, arg1)
    }
}

pub(super) unsafe fn msg3<R, A0, A1, A2>(
    receiver: &Obj,
    selector: &Sel,
    arg0: A0,
    arg1: A1,
    arg2: A2,
) -> R {
    unsafe {
        let fn_ptr = transmute::<
            unsafe extern "C" fn(),
            unsafe extern "C" fn(receiver: Ptr, selector: Ptr, arg0: A0, arg1: A1, arg2: A2) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver.get(), selector.get(), arg0, arg1, arg2)
    }
}

pub(super) unsafe fn msg4<R, A0, A1, A2, A3>(
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
