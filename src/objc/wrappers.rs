use std::{ffi::CStr, ops::BitOr};

use super::crimes::{
    Bool, CGFloat, CStrPtr, Cls, NSUInteger, Obj, Ptr, Sel, class_addMethod, msg0, msg1, msg2,
    msg4, to_imp2,
};

unsafe extern "C" {
    pub safe fn MTLCreateSystemDefaultDevice() -> MTLDevice;
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

static SEL_ALLOC: Sel = Sel::uninit();
static NS_OBJECT_CLS: Cls = Cls::uninit();
static NS_STRING_CLS: Cls = Cls::uninit();
static NS_STRING_SEL_NEW: Sel = Sel::uninit();

pub fn init_base() {
    NS_OBJECT_CLS.init(c"NSObject");
    SEL_ALLOC.init(c"alloc");
    NS_STRING_CLS.init(c"NSString");
    NS_STRING_SEL_NEW.init(c"stringWithUTF8String:");
}

#[repr(transparent)]
pub struct NSString(Obj);

impl NSString {
    pub fn new(s: &CStr) -> NSString {
        unsafe { msg1::<CStrPtr, NSString>(&NS_STRING_CLS.0, &NS_STRING_SEL_NEW, s.as_ptr()) }
    }
}

static NS_APPLICATION_CLS: Cls = Cls::uninit();
static NS_APPLICATION_SEL_SHARED_APP: Sel = Sel::uninit();
static NS_APPLICATION_SEL_SET_ACTIVATION_POLICY: Sel = Sel::uninit();
static NS_APPLICATION_SEL_RUN: Sel = Sel::uninit();
static NS_APPLICATION_SEL_TERMINATE: Sel = Sel::uninit();

static NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE: Sel = Sel::uninit();

#[repr(transparent)]
pub struct NSApplication(Obj);

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
        NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE.init(c"windowShouldClose:");

        unsafe {
            let imp = to_imp2(NSWindow::window_should_close);
            class_addMethod(
                NS_OBJECT_CLS.get(),
                NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE.get(),
                imp,
                c"v@:".as_ptr(),
            );
        }
    }

    pub fn shared_app() -> NSApplication {
        unsafe { msg0::<NSApplication>(&NS_APPLICATION_CLS.0, &NS_APPLICATION_SEL_SHARED_APP) }
    }

    pub fn set_activation_policy(&self, policy: NSApplicationActivationPolicy) {
        unsafe {
            msg1::<NSApplicationActivationPolicy, Bool>(
                &self.0,
                &NS_APPLICATION_SEL_SET_ACTIVATION_POLICY,
                policy,
            )
        };
    }

    pub fn run(&self) {
        unsafe { msg0::<Ptr>(&self.0, &NS_APPLICATION_SEL_RUN) };
    }

    pub fn terminate(&self, sender: Ptr) {
        unsafe { msg1::<Ptr, Ptr>(&self.0, &NS_APPLICATION_SEL_TERMINATE, sender) };
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
    use super::crimes::NSInteger;

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
static NS_WINDOW_SEL_INIT: Sel = Sel::uninit();
static NS_WINDOW_SEL_SET_TITLE: Sel = Sel::uninit();
static NS_WINDOW_SEL_SET_IS_VISIBLE: Sel = Sel::uninit();
static NS_WINDOW_SEL_MAKE_MAIN: Sel = Sel::uninit();
static NS_WINDOW_SEL_CENTER: Sel = Sel::uninit();

#[repr(transparent)]
pub struct NSWindow(Obj);

impl NSWindow {
    pub fn init() {
        NS_WINDOW_CLS.init(c"NSWindow");
        NS_WINDOW_SEL_INIT.init(c"initWithContentRect:styleMask:backing:defer:");
        NS_WINDOW_SEL_SET_TITLE.init(c"setTitle:");
        NS_WINDOW_SEL_SET_IS_VISIBLE.init(c"setIsVisible:");
        NS_WINDOW_SEL_MAKE_MAIN.init(c"makeMainWindow");
        NS_WINDOW_SEL_CENTER.init(c"center");
    }

    pub fn alloc_init(
        rect: CGRect,
        style_mask: NSWindowStyleMask,
        backing: NSBackingStoreType,
        defer: bool,
    ) -> NSWindow {
        unsafe {
            let obj = msg0::<Obj>(&NS_WINDOW_CLS.0, &SEL_ALLOC);
            msg4::<CGRect, NSWindowStyleMask, NSBackingStoreType, Bool, NSWindow>(
                &obj,
                &NS_WINDOW_SEL_INIT,
                rect,
                style_mask,
                backing,
                defer as Bool,
            )
        }
    }

    pub fn set_title(&self, title: NSString) {
        unsafe { msg1::<NSString, Ptr>(&self.0, &NS_WINDOW_SEL_SET_TITLE, title) };
    }

    pub fn set_visibility(&self, is_visible: bool) {
        unsafe { msg1::<Bool, Ptr>(&self.0, &NS_WINDOW_SEL_SET_IS_VISIBLE, is_visible as Bool) };
    }

    pub fn set_main(&self) {
        unsafe { msg0::<Ptr>(&self.0, &NS_WINDOW_SEL_MAKE_MAIN) };
    }

    pub fn center(&self) {
        unsafe { msg0::<Ptr>(&self.0, &NS_WINDOW_SEL_CENTER) };
    }

    fn window_should_close(sender: Ptr, _sel: Sel) -> Bool {
        NSApplication::shared_app().terminate(sender);
        true as Bool
    }
}

static MTK_VIEW_CLS: Cls = Cls::uninit();
static MTK_VIEW_SEL_INIT: Sel = Sel::uninit();

#[repr(transparent)]
pub struct MTKView(Ptr);

impl MTKView {
    pub fn init() {
        MTK_VIEW_CLS.init(c"MTKView");
        MTK_VIEW_SEL_INIT.init(c"initWithFrame:device:");
        MTL_SEL_MAX_TG_MEM_LEN.init(c"maxThreadgroupMemoryLength");
    }

    pub fn new(frame: CGRect, device: MTLDevice) -> Self {
        println!("ready?");

        bpoint();

        unsafe { msg2::<_, _, MTKView>(&MTK_VIEW_CLS.0, &MTK_VIEW_SEL_INIT, frame, device) }
    }
}

fn bpoint() {}

static MTL_SEL_MAX_TG_MEM_LEN: Sel = Sel::uninit();

#[repr(transparent)]
pub struct MTLDevice(Obj);

impl MTLDevice {
    pub fn check_null(&self) {
        self.0.get();
    }

    pub fn max_tg_mem_len(&self) -> NSUInteger {
        unsafe { msg0::<NSUInteger>(&self.0, &MTL_SEL_MAX_TG_MEM_LEN) }
    }
}
