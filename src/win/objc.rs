use std::{
    ffi::{CStr, c_char, c_double, c_longlong, c_schar, c_ulonglong, c_void},
    mem::transmute,
    ops::BitOr,
    ptr::null,
};

pub type Sel = *const c_void;
pub type Ptr = *mut c_void;
pub type Imp = unsafe extern "C" fn() -> *const c_void;
pub type CStrPtr = *const c_char;
pub type NSInteger = c_longlong;
pub type NSUInteger = c_ulonglong;
pub type Bool = c_schar;
pub type CGFloat = c_double;

unsafe extern "C" {
    unsafe fn objc_getClass(name: CStrPtr) -> Ptr;
    unsafe fn sel_registerName(name: CStrPtr) -> Sel;
    unsafe fn objc_msgSend();
    unsafe fn class_addMethod(cls: Ptr, name: Sel, imp: Imp, types: CStrPtr) -> Bool;
}

pub fn get_class(name: &CStr) -> Ptr {
    unsafe { objc_getClass(name.as_ptr()) }
}

pub fn sel(name: &CStr) -> Sel {
    unsafe { sel_registerName(name.as_ptr()) }
}

pub const unsafe fn msg0<R>() -> unsafe extern "C" fn(receiver: Ptr, selector: Sel) -> R {
    unsafe { transmute(objc_msgSend as unsafe extern "C" fn()) }
}

pub const unsafe fn msg1<A0, R>()
-> unsafe extern "C" fn(receiver: Ptr, selector: Sel, arg0: A0) -> R {
    unsafe { transmute(objc_msgSend as unsafe extern "C" fn()) }
}

pub const unsafe fn msg4<A0, A1, A2, A3, R>()
-> unsafe extern "C" fn(receiver: Ptr, selector: Sel, arg0: A0, arg1: A1, arg2: A2, arg3: A3) -> R {
    unsafe { transmute(objc_msgSend as unsafe extern "C" fn()) }
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

pub struct NSStringCls {
    cls: Ptr,
    sel: Sel,
}

impl NSStringCls {
    pub fn init() -> &'static Self {
        let cls = get_class(c"NSString");
        let sel = sel(c"stringWithUTF8String:");
        Box::leak(Box::new(NSStringCls { cls, sel }))
    }

    pub fn new(&'static self, s: &CStr) -> NSString {
        let obj = unsafe { msg1::<CStrPtr, Ptr>()(self.cls, self.sel, s.as_ptr()) };
        NSString { obj }
    }
}

pub struct NSString {
    obj: Ptr,
}

pub struct NSApplicationCls {
    cls: Ptr,
    sel_shared_app: Sel,
    sel_set_activation_policy: Sel,
    sel_run: Sel,
}

impl NSApplicationCls {
    pub fn init() -> &'static Self {
        let cls = get_class(c"NSApplication");
        let sel_shared_app = sel(c"sharedApplication");
        let sel_set_activation_policy = sel(c"setActivationPolicy:");
        let sel_run = sel(c"run");
        let default = get_class(c"NSObject");
        let sel_should_close = sel(c"windowShouldClose:");
        unsafe {
            let imp = transmute(window_should_close as fn(Ptr, Sel) -> Bool);
            class_addMethod(default, sel_should_close, imp, c"v@:".as_ptr());
        }
        Box::leak(Box::new(NSApplicationCls {
            cls,
            sel_shared_app,
            sel_set_activation_policy,
            sel_run,
        }))
    }

    pub fn shared_app(&'static self) -> NSApplication {
        let obj = unsafe { msg0::<Ptr>()(self.cls, self.sel_shared_app) };
        NSApplication { cls: self, obj }
    }
}

pub struct NSApplication {
    cls: &'static NSApplicationCls,
    obj: Ptr,
}

impl NSApplication {
    pub fn set_activation_policy(&self, policy: NSApplicationActivationPolicy) {
        unsafe {
            msg1::<NSApplicationActivationPolicy, Bool>()(
                self.obj,
                self.cls.sel_set_activation_policy,
                policy,
            )
        };
    }

    pub fn run(&self) {
        unsafe { msg0::<Ptr>()(self.obj, self.cls.sel_run) };
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

pub struct NSWindowCls {
    cls: Ptr,
    sel_alloc: Sel,
    sel_win_init: Sel,
    sel_set_title: Sel,
    sel_set_visible: Sel,
    sel_make_main: Sel,
}

impl NSWindowCls {
    pub fn init() -> &'static NSWindowCls {
        let cls = get_class(c"NSWindow");
        let sel_alloc = sel(c"alloc");
        let sel_win_init = sel(c"initWithContentRect:styleMask:backing:defer:");
        let sel_set_title = sel(c"setTitle:");
        let sel_set_visible = sel(c"setIsVisible:");
        let sel_make_main = sel(c"makeMainWindow");

        Box::leak(Box::new(Self {
            cls,
            sel_alloc,
            sel_win_init,
            sel_set_title,
            sel_set_visible,
            sel_make_main,
        }))
    }

    pub fn alloc_init(
        &'static self,
        app_cls: &'static NSApplicationCls,
        rect: CGRect,
        style_mask: NSWindowStyleMask,
        backing: NSBackingStoreType,
        defer: bool,
    ) -> NSWindow {
        let obj = unsafe {
            let obj = msg0::<Ptr>()(self.cls, self.sel_alloc);
            msg4::<CGRect, NSWindowStyleMask, NSBackingStoreType, Bool, Ptr>()(
                obj,
                self.sel_win_init,
                rect,
                style_mask,
                backing,
                defer as Bool,
            )
        };
        NSWindow { app_cls, cls: self, obj }
    }

    fn window_should_close(sender: Ptr, sel: Sel) -> Bool {
        println!("FROM window_should_close {:?} {:?}", sel, sender);
        true as Bool
    }
}

pub struct NSWindow {
    app_cls: &'static NSApplicationCls,
    cls: &'static NSWindowCls,
    obj: Ptr,
}

impl NSWindow {
    pub fn set_title(&self, title: NSString) {
        unsafe { msg1::<Ptr, Ptr>()(self.obj, self.cls.sel_set_title, title.obj) };
    }

    pub fn set_visibility(&self, is_visible: bool) {
        unsafe { msg1::<Bool, Ptr>()(self.obj, self.cls.sel_set_visible, is_visible as Bool) };
    }

    pub fn set_main(&self) {
        unsafe { msg0::<Ptr>()(self.obj, self.cls.sel_make_main) };
    }
}
