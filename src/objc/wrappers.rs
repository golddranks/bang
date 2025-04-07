#![allow(dead_code)]
use std::{ffi::CStr, fmt::Debug, ops::BitOr};

use crate::objc::crimes::objc_prop_sel_init;

use super::{
    AllocObj, InstancePtr,
    crimes::{
        Bool, CGFloat, CStrPtr, Cls, NSUInteger, Obj, Ptr, Sel, add_method2, add_method3, msg0,
        msg1, msg2, msg3, msg4, objc_instance_ptr, objc_prop_impl, objc_protocol_ptr,
    },
};

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

unsafe extern "C" {
    safe fn MTLCreateSystemDefaultDevice() -> Ptr;
}

// Custom Debug impl, so we won't use the objc_type! macro
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NSString(Obj);

objc_instance_ptr!(NSError);
objc_instance_ptr!(NSURL);
objc_instance_ptr!(NSApplication);
objc_instance_ptr!(NSWindow);
objc_instance_ptr!(MTKView);
objc_instance_ptr!(MTLRenderPassDescriptor);
objc_instance_ptr!(CAMetalDrawable);
objc_instance_ptr!(MTLRenderPipelineDescriptor);

objc_protocol_ptr!(MTLDevice);
objc_protocol_ptr!(MTLCommandQueue);
objc_protocol_ptr!(MTLCommandBuffer);
objc_protocol_ptr!(MTLRenderCommandEncoder);
objc_protocol_ptr!(MTLBuffer);
objc_protocol_ptr!(MTLRenderPipelineState);
objc_protocol_ptr!(MTLLibrary);
objc_protocol_ptr!(MTLFunction);

pub mod cls {
    use crate::objc::crimes::objc_class;

    objc_class!(NSURL);
    objc_class!(NSString);
    objc_class!(NSError);
    objc_class!(NSApplication);
    objc_class!(NSWindow);
    objc_class!(MTKView);
    objc_class!(MTLRenderPassDescriptor);
    objc_class!(CAMetalDrawable);
    objc_class!(MTLRenderPipelineDescriptor);
}

pub(super) mod sel {
    use crate::objc::crimes::{objc_prop_sel, objc_sel};

    objc_sel!(alloc);
    objc_sel!(init);
    objc_sel!(stringWithUTF8String_);
    objc_sel!(URLWithString_);
    objc_sel!(UTF8String);

    // NSApplication
    objc_sel!(sharedApplication);
    objc_sel!(setActivationPolicy_);
    objc_sel!(run);
    objc_sel!(stop_);

    // NSWindow
    objc_sel!(initWithContentRect_styleMask_backing_defer_);
    objc_sel!(makeMainWindow);
    objc_sel!(center);
    objc_sel!(windowShouldClose_);
    objc_prop_sel!(title);
    objc_prop_sel!(isVisible);
    objc_prop_sel!(contentView);

    // MKTView
    objc_sel!(initWithFrame_device_);
    objc_sel!(drawRect_);
    objc_sel!(setClearColor_);
    objc_prop_sel!(currentRenderPassDescriptor);
    objc_prop_sel!(device);
    objc_prop_sel!(currentDrawable);

    // MTLDevice
    objc_sel!(newCommandQueue);
    objc_sel!(newBufferWithBytes_length_options_);
    objc_sel!(newRenderPipelineStateWithDescriptor_error_);
    objc_sel!(newLibraryWithURL_error_);

    // MTLCommandQueue
    objc_sel!(commandBuffer);

    // MTLCommandBuffer
    objc_sel!(renderCommandEncoderWithDescriptor_);
    objc_sel!(presentDrawable_);
    objc_sel!(commit);

    // MTLCommandEncoder
    objc_sel!(endEncoding);

    // MTLRenderPipelineDescriptor
    objc_prop_sel!(vertexFunction);
    objc_prop_sel!(fragmentFunction);

    // MTLLibrary
    objc_sel!(newFunctionWithName_);

    objc_sel!(newRenderPipelineDescriptor);
    objc_sel!(setVertexFunction_);
    objc_sel!(setFragmentFunction_);
    objc_sel!(setRenderPipelineState_);
    objc_sel!(setVertexBuffer_offset_index_);
    objc_sel!(drawPrimitives_vertexStart_vertexCount_);
}

pub fn init_objc() {
    cls::NSURL.init();
    cls::NSString.init();
    cls::NSError.init();
    cls::NSApplication.init();
    cls::NSWindow.init();
    cls::MTKView.init();
    cls::MTLRenderPassDescriptor.init();
    cls::CAMetalDrawable.init();
    cls::MTLRenderPipelineDescriptor.init();

    sel::alloc.init();
    sel::init.init();
    sel::stringWithUTF8String_.init();
    sel::URLWithString_.init();
    sel::UTF8String.init();

    // NSApplication
    sel::sharedApplication.init();
    sel::setActivationPolicy_.init();
    sel::run.init();
    sel::stop_.init();

    // NSWindow
    sel::initWithContentRect_styleMask_backing_defer_.init();
    sel::makeMainWindow.init();
    sel::center.init();
    sel::windowShouldClose_.init();
    objc_prop_sel_init!(title);
    objc_prop_sel_init!(isVisible);
    objc_prop_sel_init!(contentView);

    // MTKView
    sel::initWithFrame_device_.init();
    sel::drawRect_.init();
    sel::setClearColor_.init();
    objc_prop_sel_init!(currentRenderPassDescriptor);
    objc_prop_sel_init!(device);
    objc_prop_sel_init!(currentDrawable);

    // MTLDevice
    sel::newCommandQueue.init();
    sel::newBufferWithBytes_length_options_.init();
    sel::newRenderPipelineStateWithDescriptor_error_.init();
    sel::newLibraryWithURL_error_.init();

    // MTLCommandQueue
    sel::commandBuffer.init();

    // MTLCommandBuffer
    sel::renderCommandEncoderWithDescriptor_.init();
    sel::presentDrawable_.init();
    sel::commit.init();

    // MTLCommandEncoder
    sel::endEncoding.init();

    // MTLRenderPipelineDescriptor
    objc_prop_sel_init!(vertexFunction);
    objc_prop_sel_init!(fragmentFunction);

    // MTLLibrary
    sel::newFunctionWithName_.init();

    sel::newRenderPipelineDescriptor.init();
    sel::setVertexFunction_.init();
    sel::setFragmentFunction_.init();
    sel::setRenderPipelineState_.init();
    sel::setVertexBuffer_offset_index_.init();
    sel::drawPrimitives_vertexStart_vertexCount_.init();
}

impl NSString {
    pub fn new(s: &CStr) -> NSString {
        // SAFETY: OK.
        unsafe {
            msg1::<NSString, CStrPtr>(
                cls::NSString.obj(),
                sel::stringWithUTF8String_.sel(),
                CStrPtr::new(s),
            )
        }
    }

    pub fn as_str(&self) -> &CStr {
        // SAFETY: OK. the CStrPtr lifetime is constrained by the output &CStr, which is constrained by &self
        unsafe { msg0::<CStrPtr>(self.0, sel::UTF8String.sel()) }.to_cstr()
    }
}

impl Debug for NSString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

#[test]
fn test_ns_string() {
    init_objc();
    let s = NSString::new(c"huhheiやー");
    assert_eq!(s.as_str(), c"huhheiやー");
}

impl NSApplication {
    pub fn shared() -> NSApplication {
        unsafe { msg0::<NSApplication>(cls::NSApplication.obj(), sel::sharedApplication.sel()) }
    }

    pub fn set_activation_policy(&self, policy: NSApplicationActivationPolicy) {
        unsafe {
            msg1::<Bool, NSApplicationActivationPolicy>(
                self.0,
                sel::setActivationPolicy_.sel(),
                policy,
            )
        };
    }

    pub fn run(&self) {
        unsafe { msg0::<()>(self.0, sel::run.sel()) };
    }

    pub fn stop(&self, sender: Obj) {
        unsafe { msg1::<(), Obj>(self.0, sel::stop_.sel(), sender) };
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
    pub const TITLED: Self = Self(1);
    pub const CLOSABLE: Self = Self(2);
    pub const MINIATURIZABLE: Self = Self(4);
    pub const RESIZABLE: Self = Self(8);
}

impl BitOr for NSWindowStyleMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
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

impl NSWindow {
    pub fn override_window_should_close(f: extern "C" fn(NSWindow, Sel) -> Bool) {
        unsafe {
            add_method2(
                cls::NSWindow.cls(),
                sel::windowShouldClose_.sel(),
                f,
                c"c@:@",
            );
        }
    }

    pub fn init(
        alloc: AllocObj<NSWindow>,
        rect: CGRect,
        style_mask: NSWindowStyleMask,
        backing: NSBackingStoreType,
        defer: bool,
    ) -> NSWindow {
        unsafe {
            msg4::<NSWindow, CGRect, NSWindowStyleMask, NSBackingStoreType, Bool>(
                alloc.obj(),
                sel::initWithContentRect_styleMask_backing_defer_.sel(),
                rect,
                style_mask,
                backing,
                defer as Bool,
            )
        }
    }

    pub fn set_main(&self) {
        unsafe { msg0::<()>(self.0, sel::makeMainWindow.sel()) };
    }

    pub fn center(&self) {
        unsafe { msg0::<()>(self.0, sel::center.sel()) };
    }

    objc_prop_impl!(title, NSString, title, set_title);
    objc_prop_impl!(isVisible, bool, is_visible, set_is_visible);
    objc_prop_impl!(contentView, MTKView, content_view, set_content_view);
}

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

impl MTKView {
    pub fn override_draw_rect(f: extern "C" fn(MTKView, Sel, CGRect)) -> bool {
        unsafe {
            add_method3(
                cls::MTKView.cls(),
                sel::drawRect_.sel(),
                f,
                c"v@:{CGRect={CGPoint=dd}{CGSize=dd}}",
            )
        }
    }

    objc_prop_impl!(
        currentRenderPassDescriptor,
        Option<MTLRenderPassDescriptor>,
        current_rendpass_desc,
        set_current_rendpass_desc
    );

    objc_prop_impl!(
        currentDrawable,
        Option<CAMetalDrawable>,
        current_drawable,
        set_current_drawable
    );

    objc_prop_impl!(device, Option<MTLDevice>, device, set_device);

    pub fn init(alloc: AllocObj<MTKView>, frame: CGRect, device: MTLDevice) -> Self {
        unsafe {
            msg2::<MTKView, CGRect, MTLDevice>(
                alloc.obj(),
                sel::initWithFrame_device_.sel(),
                frame,
                device,
            )
        }
    }
}

impl MTLDevice {
    pub fn get_default() -> MTLDevice {
        let ptr = MTLCreateSystemDefaultDevice();
        MTLDevice(Obj::new(ptr))
    }

    pub fn new_cmd_queue(&self) -> Option<MTLCommandQueue> {
        unsafe { msg0::<Option<MTLCommandQueue>>(self.0, sel::newCommandQueue.sel()) }
    }

    pub fn new_buf<T>(&self, bytes: &[T]) -> Option<MTLBuffer> {
        unsafe {
            msg3::<Option<MTLBuffer>, *const u8, NSUInteger, MTLResourceOptions>(
                self.0,
                sel::newBufferWithBytes_length_options_.sel(),
                bytes.as_ptr() as *const u8,
                (bytes.len() * size_of::<T>()) as NSUInteger,
                MTLResourceOptions::DEFAULT,
            )
        }
    }

    pub fn new_rend_pl_state(
        &self,
        desc: MTLRenderPipelineDescriptor,
    ) -> Result<MTLRenderPipelineState, NSError> {
        let mut error = None;
        let res = unsafe {
            msg2::<Option<MTLRenderPipelineState>, MTLRenderPipelineDescriptor, &mut Option<NSError>>(
                self.0,
                sel::newRenderPipelineStateWithDescriptor_error_.sel(),
                desc,
                &mut error,
            )
        };
        match (res, error) {
            (Some(state), None) => Ok(state),
            (None, Some(err)) => Err(err),
            (None, None) | (Some(_), Some(_)) => unreachable!(),
        }
    }

    pub fn new_lib_with_url(&self, url: NSURL) -> Result<MTLLibrary, NSError> {
        let mut error = None;
        let res = unsafe {
            msg2::<Option<MTLLibrary>, NSURL, &mut Option<NSError>>(
                self.0,
                sel::newLibraryWithURL_error_.sel(),
                url,
                &mut error,
            )
        };
        match (res, error) {
            (Some(lib), None) => Ok(lib),
            (None, Some(err)) => Err(err),
            (None, None) | (Some(_), Some(_)) => unreachable!(),
        }
    }
}

impl MTLCommandQueue {
    pub fn cmd_buf(&self) -> Option<MTLCommandBuffer> {
        unsafe { msg0::<Option<MTLCommandBuffer>>(self.0, sel::commandBuffer.sel()) }
    }
}

impl MTLCommandBuffer {
    pub fn rencoder_with_desc(
        &self,
        pass_desc: MTLRenderPassDescriptor,
    ) -> Option<MTLRenderCommandEncoder> {
        unsafe {
            msg1::<Option<MTLRenderCommandEncoder>, MTLRenderPassDescriptor>(
                self.0,
                sel::renderCommandEncoderWithDescriptor_.sel(),
                pass_desc,
            )
        }
    }

    pub fn present_drawable(&self, drawable: CAMetalDrawable) {
        unsafe { msg1::<(), CAMetalDrawable>(self.0, sel::presentDrawable_.sel(), drawable) }
    }

    pub fn commit(&self) {
        unsafe { msg0::<()>(self.0, sel::commit.sel()) }
    }
}

impl MTLRenderCommandEncoder {
    pub fn end(&self) {
        unsafe { msg0::<()>(self.0, sel::endEncoding.sel()) }
    }
}

#[repr(transparent)]
pub struct MTLResourceOptions(NSUInteger);

impl MTLResourceOptions {
    pub const DEFAULT: Self = Self(0);
}

impl BitOr for MTLResourceOptions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl MTLRenderPipelineDescriptor {
    pub fn new() -> Self {
        let alloc = MTLRenderPipelineDescriptor::alloc();
        unsafe { msg0::<MTLRenderPipelineDescriptor>(alloc.obj(), sel::init.sel()) }
    }

    objc_prop_impl!(vertexFunction, MTLFunction, vtex_fn, set_vtex_fn);
    objc_prop_impl!(fragmentFunction, MTLFunction, frag_fn, set_frag_fn);
}

impl NSURL {
    pub fn new(s: &CStr) -> NSURL {
        // SAFETY: OK.
        unsafe {
            msg1::<NSURL, NSString>(
                cls::NSURL.obj(),
                sel::URLWithString_.sel(),
                NSString::new(s),
            )
        }
    }
}

impl MTLLibrary {
    pub fn new_fn(&self, name: &CStr) -> MTLFunction {
        // SAFETY: OK.
        unsafe {
            msg1::<MTLFunction, NSString>(
                self.0,
                sel::newFunctionWithName_.sel(),
                NSString::new(name),
            )
        }
    }
}
