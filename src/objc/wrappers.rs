use std::{ffi::CStr, fmt::Debug, ops::BitOr, ptr::null_mut};

use std::concat_idents;

use crate::objc::crimes::msg3;

use super::crimes::{
    Bool, CGFloat, CStrPtr, Cls, NSUInteger, Obj, Ptr, Sel, add_method2, add_method3, msg0, msg1,
    msg2, msg4, register_class, subclass,
};

unsafe extern "C" {
    safe fn MTLCreateSystemDefaultDevice() -> Ptr;
}

#[derive(Debug)]
#[repr(C)]
pub struct CGPoint {
    pub x: CGFloat,
    pub y: CGFloat,
}

#[derive(Debug)]
#[repr(C)]
pub struct CGSize {
    pub width: CGFloat,
    pub height: CGFloat,
}

#[derive(Debug)]
#[repr(C)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

static SEL_ALLOC: Sel = Sel::uninit();
static NS_STRING_CLS: Cls = Cls::uninit();
static NS_STRING_SEL_NEW: Sel = Sel::uninit();
static NS_STRING_SEL_UTF8: Sel = Sel::uninit();

pub fn init_base() {
    SEL_ALLOC.init(c"alloc");
    NS_STRING_CLS.init(c"NSString");
    NS_STRING_SEL_NEW.init(c"stringWithUTF8String:");
    NS_STRING_SEL_UTF8.init(c"UTF8String");
}

fn alloc(cls: &Cls) -> Obj {
    unsafe { msg0::<Obj>(&cls.0, &SEL_ALLOC) }
}

#[repr(transparent)]
pub struct NSString(Obj);

impl NSString {
    pub fn new(s: &CStr) -> NSString {
        unsafe { msg1::<NSString, CStrPtr>(&NS_STRING_CLS.0, &NS_STRING_SEL_NEW, s.as_ptr()) }
    }

    pub fn as_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(msg0::<CStrPtr>(&self.0, &NS_STRING_SEL_UTF8)) }
    }
}

impl Debug for NSString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

#[test]
fn test_ns_string() {
    init_base();
    let s = NSString::new(c"huhheiやー");
    assert_eq!(s.as_str(), c"huhheiやー");
}

static NS_APPLICATION_CLS: Cls = Cls::uninit();
static NS_APPLICATION_SEL_SHARED_APP: Sel = Sel::uninit();
static NS_APPLICATION_SEL_SET_ACTIVATION_POLICY: Sel = Sel::uninit();
static NS_APPLICATION_SEL_RUN: Sel = Sel::uninit();
static NS_APPLICATION_SEL_STOP: Sel = Sel::uninit();

#[repr(transparent)]
pub struct NSApplication(Obj);

impl NSApplication {
    pub fn init() {
        NS_APPLICATION_CLS.init(c"NSApplication");
        NS_APPLICATION_SEL_SHARED_APP.init(c"sharedApplication");
        NS_APPLICATION_SEL_SET_ACTIVATION_POLICY.init(c"setActivationPolicy:");
        NS_APPLICATION_SEL_RUN.init(c"run");
        NS_APPLICATION_SEL_STOP.init(c"stop:");
    }

    pub fn shared_app() -> NSApplication {
        unsafe { msg0::<NSApplication>(&NS_APPLICATION_CLS.0, &NS_APPLICATION_SEL_SHARED_APP) }
    }

    pub fn set_activation_policy(&self, policy: NSApplicationActivationPolicy) {
        unsafe {
            msg1::<Bool, NSApplicationActivationPolicy>(
                &self.0,
                &NS_APPLICATION_SEL_SET_ACTIVATION_POLICY,
                policy,
            )
        };
    }

    pub fn run(&self) {
        unsafe { msg0::<Ptr>(&self.0, &NS_APPLICATION_SEL_RUN) };
    }

    pub fn stop(&self, sender: Ptr) {
        unsafe { msg1::<Ptr, Ptr>(&self.0, &NS_APPLICATION_SEL_STOP, sender) };
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
    type NSInteger = std::ffi::c_longlong;

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
static NS_WINDOW_SEL_SET_CONTENT_VIEW: Sel = Sel::uninit();
static NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE: Sel = Sel::uninit();

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
        NS_WINDOW_SEL_SET_CONTENT_VIEW.init(c"setContentView:");
        NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE.init(c"windowShouldClose:");

        unsafe {
            add_method2(
                &NS_WINDOW_CLS,
                &NS_OBJECT_SEL_WINDOW_SHOULD_CLOSE,
                NSWindow::window_should_close_override,
                c"c@:@",
            );
        }
    }

    pub fn alloc_init(
        rect: CGRect,
        style_mask: NSWindowStyleMask,
        backing: NSBackingStoreType,
        defer: bool,
    ) -> NSWindow {
        unsafe {
            msg4::<NSWindow, CGRect, NSWindowStyleMask, NSBackingStoreType, Bool>(
                &alloc(&NS_WINDOW_CLS),
                &NS_WINDOW_SEL_INIT,
                rect,
                style_mask,
                backing,
                defer as Bool,
            )
        }
    }

    pub fn set_title(&self, title: NSString) {
        unsafe { msg1::<Ptr, NSString>(&self.0, &NS_WINDOW_SEL_SET_TITLE, title) };
    }

    pub fn set_visibility(&self, is_visible: bool) {
        unsafe { msg1::<Ptr, Bool>(&self.0, &NS_WINDOW_SEL_SET_IS_VISIBLE, is_visible as Bool) };
    }

    pub fn set_main(&self) {
        unsafe { msg0::<Ptr>(&self.0, &NS_WINDOW_SEL_MAKE_MAIN) };
    }

    pub fn center(&self) {
        unsafe { msg0::<Ptr>(&self.0, &NS_WINDOW_SEL_CENTER) };
    }

    extern "C" fn window_should_close_override(sender: Ptr, _sel: Sel) -> Bool {
        println!("Window closed!");
        NSApplication::shared_app().stop(sender);
        true as Bool
    }

    pub fn set_content_view(&self, view: MTKView) {
        unsafe { msg1::<Ptr, MTKView>(&self.0, &NS_WINDOW_SEL_SET_CONTENT_VIEW, view) };
    }
}

static MTK_VIEW_CLS_CUSTOM: Cls = Cls::uninit();
static MTK_VIEW_SEL_INIT: Sel = Sel::uninit();
static MTK_VIEW_SEL_DRAW_RECT: Sel = Sel::uninit();
static MTK_VIEW_SEL_DEVICE: Sel = Sel::uninit();
static MTK_VIEW_SEL_CURRENT_DRAWABLE: Sel = Sel::uninit();
static MTK_VIEW_SEL_RPASS_DESC: Sel = Sel::uninit();

static MTL_DEVICE_SEL_NEW_CMD_QUEUE: Sel = Sel::uninit();
static MTL_CMD_BUF_SEL_CMD_BUF: Sel = Sel::uninit();
static MTL_CMD_BUF_SEL_RENCODER: Sel = Sel::uninit();
static MTK_VIEW_SEL_SET_CLEAR_COLOR: Sel = Sel::uninit();
static MTL_RENCODER_SEL_END: Sel = Sel::uninit();
static MTL_CMD_BUF_SEL_PRESENT_DRAWABLE: Sel = Sel::uninit();
static MTL_CMD_BUF_SEL_COMMIT: Sel = Sel::uninit();
static MTL_DEVICE_SEL_NEW_BUF: Sel = Sel::uninit();

static MTL_DEVICE_SEL_NEW_LIB: Sel = Sel::uninit();
static MTL_DEVICE_SEL_NEW_PIPE_DESC: Sel = Sel::uninit();
static MTL_PIPE_DESC_SEL_ADD_VTEX_FN: Sel = Sel::uninit();
static MTL_PIPE_DESC_SEL_ADD_FRAG_FN: Sel = Sel::uninit();
static MTL_DEVICE_SEL_NEW_PIPE_STATE: Sel = Sel::uninit();
static MTL_RENCODER_SEL_SET_PIPE_STATE: Sel = Sel::uninit();
static MTL_RENCODER_SEL_SET_VTEX_BUF: Sel = Sel::uninit();
static MTL_RENCODER_SEL_DRAW_PRIMITIVES: Sel = Sel::uninit();

#[repr(transparent)]
pub struct MTKView(Obj);

#[repr(C)]
#[derive(Debug)]
struct MTLClearColor {
    red: f64,
    green: f64,
    blue: f64,
    alpha: f64,
}

#[repr(i64)]
#[derive(Debug)]
enum MTLPrimitiveType {
    Triangle = 3,
}

macro_rules! c_stringify {
    ($str:expr) => {
        const {
            match CStr::from_bytes_with_nul(concat!(stringify!($str), "\0").as_bytes()) {
                Ok(cstr) => cstr,
                Err(_) => unreachable!(),
            }
        }
    };
}

macro_rules! objc_class {
    ($class:ident) => {
        #[allow(nonstandard_style)]
        static $class: Cls = Cls::uninit();
        $class.init(c_stringify!($class));
    };
}

macro_rules! objc_sel {
    ( $sel:ident ) => {
        #[allow(nonstandard_style)]
        static $sel: Sel = Sel::uninit();
        $sel.init_from_underscored_literal(stringify!($sel));
    };
}

impl MTKView {
    pub fn init() {
        objc_class!(MTKView);
        objc_sel!(initWithFrame_device_);
        MTK_VIEW_SEL_INIT.init(c"initWithFrame:device:");
        MTK_VIEW_SEL_DRAW_RECT.init(c"drawRect:");
        MTK_VIEW_SEL_DEVICE.init(c"device");
        MTK_VIEW_SEL_CURRENT_DRAWABLE.init(c"currentDrawable");
        MTK_VIEW_SEL_RPASS_DESC.init(c"currentRenderPassDescriptor");
        MTK_VIEW_SEL_SET_CLEAR_COLOR.init(c"setClearColor:");

        MTL_DEVICE_SEL_NEW_CMD_QUEUE.init(c"newCommandQueue");
        MTL_CMD_BUF_SEL_CMD_BUF.init(c"commandBuffer");
        MTL_CMD_BUF_SEL_RENCODER.init(c"renderCommandEncoderWithDescriptor:");
        MTL_RENCODER_SEL_END.init(c"endEncoding");
        MTL_CMD_BUF_SEL_PRESENT_DRAWABLE.init(c"presentDrawable:");
        MTL_CMD_BUF_SEL_COMMIT.init(c"commit");
        MTL_DEVICE_SEL_NEW_BUF.init(c"newBufferWithLength:options:");
        MTL_DEVICE_SEL_NEW_LIB.init(c"newLibraryWithSource:options:error:");
        MTL_DEVICE_SEL_NEW_PIPE_DESC.init(c"newRenderPipelineDescriptor");
        MTL_PIPE_DESC_SEL_ADD_VTEX_FN.init(c"setVertexFunction:");
        MTL_PIPE_DESC_SEL_ADD_FRAG_FN.init(c"setFragmentFunction:");
        MTL_DEVICE_SEL_NEW_PIPE_STATE.init(c"newRenderPipelineStateWithDescriptor:error:");
        MTL_RENCODER_SEL_SET_PIPE_STATE.init(c"setRenderPipelineState:");
        MTL_RENCODER_SEL_SET_VTEX_BUF.init(c"setVertexBuffer:offset:index:");
        MTL_RENCODER_SEL_DRAW_PRIMITIVES.init(c"drawPrimitives:vertexStart:vertexCount:");

        let custom_view = subclass(&MTKView, c"CustomMTKView");
        unsafe {
            add_method3(
                &custom_view,
                &MTK_VIEW_SEL_DRAW_RECT,
                MTKView::draw_rect_override,
                c"v@:{CGRect={CGPoint=dd}{CGSize=dd}}",
            );
        }
        register_class(&custom_view);
        MTK_VIEW_CLS_CUSTOM.init_with(custom_view);
    }

    extern "C" fn draw_rect_override(view: Obj, _sel: Sel, _dirty_rect: CGRect) {
        unsafe {
            let device = msg0::<Obj>(&view, &MTK_VIEW_SEL_DEVICE).or_die("device");
            let pass_desc = msg0::<Obj>(&view, &MTK_VIEW_SEL_RPASS_DESC).or_die("pass_dec");
            let drawable = msg0::<Obj>(&view, &MTK_VIEW_SEL_CURRENT_DRAWABLE).or_die("drawable");
            let cmd_queue = msg0::<Obj>(&device, &MTL_DEVICE_SEL_NEW_CMD_QUEUE).or_die("cmd_queue");
            let cmd_buf = msg0::<Obj>(&cmd_queue, &MTL_CMD_BUF_SEL_CMD_BUF).or_die("cmd_buf");
            /*
            let vertices: [f32; 9] = [
                0.0, 0.5, 0.0, // Top vertex
                -0.5, -0.5, 0.0, // Bottom-left vertex
                0.5, -0.5, 0.0, // Bottom-right vertex
            ];
            let vtex_buf =
                msg1::<Obj, _>(&device, &MTL_DEVICE_SEL_NEW_BUF, &vertices).or_die("vtexu buf");
            let vtex_fn = msg3::<Obj, NSString, Ptr, Ptr>(
                &device,
                &MTL_DEVICE_SEL_NEW_LIB,
                NSString::new(VTEX_SHADER),
                null_mut(),
                null_mut(),
            )
            .or_die("vtex_fn");
            let frag_fn = msg3::<Obj, NSString, Ptr, Ptr>(
                &device,
                &MTL_DEVICE_SEL_NEW_LIB,
                NSString::new(FRAG_SHADER),
                null_mut(),
                null_mut(),
            )
            .or_die("frag_fn"); */
            let rencoder =
                msg1::<Obj, Obj>(&cmd_buf, &MTL_CMD_BUF_SEL_RENCODER, pass_desc).or_die("rencoder");
            //let pipe_desc = msg0::<Obj>(&device, &MTL_DEVICE_SEL_NEW_PIPE_DESC).or_die("pipe_desc");
            //msg1::<(), _>(&pipe_desc, &MTL_PIPE_DESC_SEL_ADD_VTEX_FN, vtex_fn);
            //msg1::<(), _>(&pipe_desc, &MTL_PIPE_DESC_SEL_ADD_FRAG_FN, frag_fn);
            //let pipe_state = msg1::<Obj, _>(&device, &MTL_DEVICE_SEL_NEW_PIPE_STATE, pipe_desc);

            msg0::<()>(&rencoder, &MTL_RENCODER_SEL_END);

            //msg1::<(), _>(&rencoder, &MTL_RENCODER_SEL_SET_PIPE_STATE, pipe_state);
            //msg1::<(), _>(&rencoder, &MTL_RENCODER_SEL_SET_VTEX_BUF, vtex_buf);
            /*msg3::<(), MTLPrimitiveType, NSUInteger, NSUInteger>(
                &rencoder,
                &MTL_RENCODER_SEL_DRAW_PRIMITIVES,
                MTLPrimitiveType::Triangle,
                0,
                3,
            );*/

            msg1::<(), _>(&cmd_buf, &MTL_CMD_BUF_SEL_PRESENT_DRAWABLE, drawable);
            msg0::<()>(&cmd_buf, &MTL_CMD_BUF_SEL_COMMIT);

            println!("draw_rect_override completed!");
        }
    }

    pub fn new(frame: CGRect, device: MTLDevice) -> Self {
        let view = unsafe {
            msg2::<MTKView, CGRect, MTLDevice>(
                &alloc(&MTK_VIEW_CLS_CUSTOM),
                &MTK_VIEW_SEL_INIT,
                frame,
                device,
            )
        };

        unsafe {
            msg1::<Ptr, MTLClearColor>(
                &view.0,
                &MTK_VIEW_SEL_SET_CLEAR_COLOR,
                MTLClearColor {
                    red: 0.5,
                    green: 0.5,
                    blue: 0.5,
                    alpha: 0.5,
                },
            );
        }
        view
    }
}

#[repr(transparent)]
pub struct MTLDevice(Obj);

impl MTLDevice {
    pub fn get_default() -> MTLDevice {
        let ptr = MTLCreateSystemDefaultDevice();
        MTLDevice(Obj::new(ptr))
    }
}

const VTEX_SHADER: &CStr = cr##"
#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float4 position [[attribute(0)]];
};

struct VertexOut {
    float4 position [[position]];
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]]) {
    VertexOut out;
    out.position = in.position; // Simply pass through the position
    return out;
}
\0"##;

const FRAG_SHADER: &CStr = cr##"
#include <metal_stdlib>
using namespace metal;

fragment float4 fragment_main() {
    return float4(1.0, 0.0, 0.0, 1.0); // Red color for the triangle
}
\0"##;
