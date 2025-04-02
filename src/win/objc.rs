use std::{
    ffi::{CStr, c_char, c_double, c_longlong, c_schar, c_ulonglong, c_void},
    mem::transmute,
    ops::{BitOr, Not},
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

pub type Ptr = *mut c_void;
pub type Imp = unsafe extern "C" fn() -> *const c_void;
pub type CStrPtr = *const c_char;
pub type NSInteger = c_longlong;
pub type NSUInteger = c_ulonglong;
pub type Bool = c_schar;
pub type CGFloat = c_double;

unsafe extern "C" {
    unsafe fn objc_getClass(name: CStrPtr) -> Ptr;
    unsafe fn sel_registerName(name: CStrPtr) -> Ptr;
    unsafe fn objc_msgSend();
    unsafe fn class_addMethod(cls: Ptr, name: Ptr, imp: Imp, types: CStrPtr) -> Bool;
}

#[repr(transparent)]
struct Cls(AtomicPtr<c_void>);

impl Cls {
    const fn uninit() -> Self {
        Self(AtomicPtr::new(null_mut()))
    }

    fn get(&self) -> Ptr {
        let ptr = self.0.load(Ordering::Relaxed);
        assert!(ptr.is_null().not());
        ptr
    }

    fn init(&self, name: &CStr) {
        let ptr = unsafe { objc_getClass(name.as_ptr()) };
        self.0.store(ptr, Ordering::Relaxed);
    }
}

#[repr(transparent)]
struct Sel(AtomicPtr<c_void>);

impl Sel {
    const fn uninit() -> Self {
        Self(AtomicPtr::new(null_mut()))
    }

    fn get(&self) -> Ptr {
        let ptr = self.0.load(Ordering::Relaxed);
        assert!(ptr.is_null().not());
        ptr
    }

    fn init(&self, name: &CStr) {
        let ptr = unsafe { sel_registerName(name.as_ptr()) };
        self.0.store(ptr, Ordering::Relaxed);
    }
}

unsafe fn msg0<R>(receiver: Ptr, selector: &Sel) -> R {
    unsafe {
        let fn_ptr = transmute::<_, unsafe extern "C" fn(receiver: Ptr, selector: Ptr) -> R>(
            objc_msgSend as unsafe extern "C" fn(),
        );
        fn_ptr(receiver, selector.get())
    }
}

unsafe fn msg1<A0, R>(receiver: Ptr, selector: &Sel, arg0: A0) -> R {
    unsafe {
        let fn_ptr = transmute::<
            _,
            unsafe extern "C" fn(receiver: Ptr, selector: Ptr, arg0: A0) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver, selector.get(), arg0)
    }
}

unsafe fn msg4<A0, A1, A2, A3, R>(
    receiver: Ptr,
    selector: &Sel,
    arg0: A0,
    arg1: A1,
    arg2: A2,
    arg3: A3,
) -> R {
    unsafe {
        let fn_ptr = transmute::<
            _,
            unsafe extern "C" fn(
                receiver: Ptr,
                selector: Ptr,
                arg0: A0,
                arg1: A1,
                arg2: A2,
                arg3: A3,
            ) -> R,
        >(objc_msgSend as unsafe extern "C" fn());
        fn_ptr(receiver, selector.get(), arg0, arg1, arg2, arg3)
    }
}

#[repr(C)]
pub struct CGPoint {
    pub x: CGFloat,
    pub y: CGFloat,
}

#[repr(C)]
pub struct CGSize {
    pub width: CGFloat,
    pub height: CGFloat,
}

#[repr(C)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

static NS_STRING_CLS: Cls = Cls::uninit();
static NS_STRING_SEL_NEW: Sel = Sel::uninit();

#[repr(transparent)]
pub struct NSString(Ptr);

impl NSString {
    pub fn init() {
        NS_STRING_CLS.init(c"NSString");
        NS_STRING_SEL_NEW.init(c"stringWithUTF8String:");
    }

    pub fn new(s: &CStr) -> NSString {
        let obj =
            unsafe { msg1::<CStrPtr, Ptr>(NS_STRING_CLS.get(), &NS_STRING_SEL_NEW, s.as_ptr()) };
        NSString(obj)
    }
}

static NS_APPLICATION_CLS: Cls = Cls::uninit();
static NS_APPLICATION_SEL_SHARED_APP: Sel = Sel::uninit();
static NS_APPLICATION_SEL_SET_ACTIVATION_POLICY: Sel = Sel::uninit();
static NS_APPLICATION_SEL_RUN: Sel = Sel::uninit();
static NS_APPLICATION_SEL_TERMINATE: Sel = Sel::uninit();

static NS_OBJECT_CLS: Cls = Cls::uninit();
static NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE: Sel = Sel::uninit();

#[repr(transparent)]
pub struct NSApplication(Ptr);

impl NSApplication {
    pub fn init() {
        NS_APPLICATION_CLS.init(c"NSApplication");
        NS_APPLICATION_SEL_SHARED_APP.init(c"sharedApplication");
        NS_APPLICATION_SEL_SET_ACTIVATION_POLICY.init(c"setActivationPolicy:");
        NS_APPLICATION_SEL_RUN.init(c"run");
        NS_APPLICATION_SEL_TERMINATE.init(c"terminate:");
        Self::init_should_close();
    }

    fn init_should_close() {
        NS_OBJECT_CLS.init(c"NSObject");
        NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE.init(c"windowShouldClose:");

        unsafe {
            let imp = transmute(NSWindow::window_should_close as fn(Ptr, Sel) -> Bool);
            class_addMethod(
                NS_OBJECT_CLS.get(),
                NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE.get(),
                imp,
                c"v@:".as_ptr(),
            );
        }
    }

    pub fn shared_app() -> NSApplication {
        let obj = unsafe { msg0::<Ptr>(NS_APPLICATION_CLS.get(), &NS_APPLICATION_SEL_SHARED_APP) };
        NSApplication(obj)
    }

    pub fn set_activation_policy(&self, policy: NSApplicationActivationPolicy) {
        unsafe {
            msg1::<NSApplicationActivationPolicy, Bool>(
                self.0,
                &NS_APPLICATION_SEL_SET_ACTIVATION_POLICY,
                policy,
            )
        };
    }

    pub fn run(&self) {
        unsafe { msg0::<Ptr>(self.0, &NS_APPLICATION_SEL_RUN) };
    }

    pub fn terminate(&self, sender: Ptr) {
        unsafe { msg1::<Ptr, Ptr>(self.0, &NS_APPLICATION_SEL_TERMINATE, sender) };
    }
}

#[repr(i64)]
pub enum NSApplicationActivationPolicy {
    Regular = 0,
}

#[repr(i64)]
pub enum NSBackingStoreType {
    Buffered = 2,
}

#[repr(transparent)]
pub struct NSWindowStyleMask(NSUInteger);

impl NSWindowStyleMask {
    pub const TITLED: Self = NSWindowStyleMask(1);
    pub const CLOSABLE: Self = NSWindowStyleMask(2);
    pub const MINIATURIZABLE: Self = NSWindowStyleMask(4);
    pub const RESIZABLE: Self = NSWindowStyleMask(8);
}

impl BitOr for NSWindowStyleMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        NSWindowStyleMask(self.0 | rhs.0)
    }
}

#[test]
fn test_mem_layout() {
    assert_eq!(
        size_of::<NSApplicationActivationPolicy>(),
        size_of::<NSInteger>()
    );
    assert_eq!(
        align_of::<NSApplicationActivationPolicy>(),
        align_of::<NSInteger>()
    );
    assert_eq!(size_of::<NSBackingStoreType>(), size_of::<NSUInteger>());
    assert_eq!(align_of::<NSBackingStoreType>(), align_of::<NSUInteger>());
}

static NS_WINDOW_CLS: Cls = Cls::uninit();
static NS_WINDOW_SEL_ALLOC: Sel = Sel::uninit();
static NS_WINDOW_SEL_INIT: Sel = Sel::uninit();
static NS_WINDOW_SEL_SET_TITLE: Sel = Sel::uninit();
static NS_WINDOW_SEL_SET_IS_VISIBLE: Sel = Sel::uninit();
static NS_WINDOW_SEL_MAKE_MAIN: Sel = Sel::uninit();

#[repr(transparent)]
pub struct NSWindow(Ptr);

impl NSWindow {
    pub fn init() {
        NS_WINDOW_CLS.init(c"NSWindow");
        NS_WINDOW_SEL_ALLOC.init(c"alloc");
        NS_WINDOW_SEL_INIT.init(c"initWithContentRect:styleMask:backing:defer:");
        NS_WINDOW_SEL_SET_TITLE.init(c"setTitle:");
        NS_WINDOW_SEL_SET_IS_VISIBLE.init(c"setIsVisible:");
        NS_WINDOW_SEL_MAKE_MAIN.init(c"makeMainWindow");
    }

    pub fn alloc_init(
        rect: CGRect,
        style_mask: NSWindowStyleMask,
        backing: NSBackingStoreType,
        defer: bool,
    ) -> NSWindow {
        let obj = unsafe {
            let obj = msg0::<Ptr>(NS_WINDOW_CLS.get(), &NS_WINDOW_SEL_ALLOC);
            msg4::<CGRect, NSWindowStyleMask, NSBackingStoreType, Bool, Ptr>(
                obj,
                &NS_WINDOW_SEL_INIT,
                rect,
                style_mask,
                backing,
                defer as Bool,
            )
        };
        NSWindow(obj)
    }

    pub fn set_title(&self, title: NSString) {
        unsafe { msg1::<NSString, Ptr>(self.0, &NS_WINDOW_SEL_SET_TITLE, title) };
    }

    pub fn set_visibility(&self, is_visible: bool) {
        unsafe { msg1::<Bool, Ptr>(self.0, &NS_WINDOW_SEL_SET_IS_VISIBLE, is_visible as Bool) };
    }

    pub fn set_main(&self) {
        unsafe { msg0::<Ptr>(self.0, &NS_WINDOW_SEL_MAKE_MAIN) };
    }

    fn window_should_close(sender: Ptr, _sel: Sel) -> Bool {
        NSApplication::shared_app().terminate(sender);
        true as Bool
    }
}
