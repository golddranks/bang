#![allow(dead_code)]
use std::{
    error::Error,
    ffi::{CStr, c_void},
    fmt::{Debug, Display},
    ops::{Add, Sub},
};

use bang_core::draw::AsBytes;

use crate::objc::crimes::objc_prop_sel_init;

use super::{
    TypedPtr,
    crimes::{
        AllocObj, Bool, CGFloat, CPtr, InstancePtr, NSInteger, NSString, NSUInteger, OPtr,
        Protocol, Ptr, Sel, TypedCls, TypedObj, derive_BitOr, init_objc_core, msg0, msg1, msg2,
        msg3, msg4, objc_class, objc_prop_impl, objc_protocol, sel::init,
    },
};

unsafe extern "C" {
    safe fn MTLCreateSystemDefaultDevice() -> Ptr;
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CGPoint {
    pub x: CGFloat,
    pub y: CGFloat,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CGSize {
    pub width: CGFloat,
    pub height: CGFloat,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CGRect {
    pub origin: CGPoint,
    pub size: CGSize,
}

objc_class!(NSError);
objc_class!(NSUrl, NSURL, (Debug, Clone, Copy));
objc_class!(NSApplication);
objc_class!(NSWindow);
objc_class!(MTKView);
objc_class!(MTLRenderPassDescriptor);
objc_class!(CAMetalDrawable);
objc_class!(MTLRenderPipelineDescriptor);
objc_class!(MTLRenderPipelineColorAttachmentDescriptorArray);
objc_class!(MTLRenderPipelineColorAttachmentDescriptor);
objc_class!(MTLCompileOptions);
objc_class!(NSEvent);
objc_class!(NSMenu);
objc_class!(NSMenuItem);
objc_class!(NSProcessInfo);
objc_class!(MTLVertexDescriptor);
objc_class!(MTLVertexAttributeDescriptor);
objc_class!(MTLVertexAttributeDescriptorArray);
objc_class!(MTLVertexBufferLayoutDescriptor);
objc_class!(MTLVertexBufferLayoutDescriptorArray);
objc_class!(MTLTextureDescriptor);

objc_protocol!(MTLDevice);
objc_protocol!(MTLCommandQueue);
objc_protocol!(MTLCommandBuffer);
objc_protocol!(MTLRenderCommandEncoder);
objc_protocol!(MTLBuffer);
objc_protocol!(MTLRenderPipelineState);
objc_protocol!(MTLLibrary);
objc_protocol!(MTLFunction);
objc_protocol!(NSApplicationDelegate);
objc_protocol!(NSWindowDelegate);
objc_protocol!(MTKViewDelegate);
objc_protocol!(MTLTexture);

pub mod sel {
    use crate::objc::crimes::{objc_prop_sel, objc_sel};

    // misc
    objc_prop_sel!(delegate);
    objc_prop_sel!(title);

    // NSError
    objc_prop_sel!(code);
    objc_prop_sel!(domain);
    objc_prop_sel!(localizedDescription);

    // NSUrl
    objc_sel!(URLWithString_);

    // NSApplication
    objc_sel!(sharedApplication);
    objc_sel!(setActivationPolicy_);
    objc_sel!(run);
    objc_sel!(stop_);
    objc_sel!(terminate_);
    objc_prop_sel!(mainMenu);
    objc_prop_sel!(isRunning);

    // NSWindow
    objc_sel!(initWithContentRect_styleMask_backing_defer_);
    objc_sel!(makeMainWindow);
    objc_sel!(center);
    objc_sel!(setContentSize_);
    objc_prop_sel!(isVisible);
    objc_prop_sel!(contentView);
    objc_prop_sel!(contentAspectRatio);
    objc_prop_sel!(contentResizeIncrements);
    objc_prop_sel!(contentLayoutRect);
    objc_prop_sel!(frame);
    objc_prop_sel!(contentMinSize);

    // MKTView
    objc_sel!(initWithFrame_device_);
    objc_sel!(drawRect_);
    objc_prop_sel!(clearColor);
    objc_prop_sel!(currentRenderPassDescriptor);
    objc_prop_sel!(device);
    objc_prop_sel!(currentDrawable);
    objc_prop_sel!(colorPixelFormat);
    objc_prop_sel!(preferredFramesPerSecond);

    // MTKViewDelegate
    objc_sel!(drawInMTKView_);
    objc_sel!(mtkView_drawableSizeWillChange_);

    // MTLDevice
    objc_sel!(newCommandQueue);
    objc_sel!(newBufferWithBytes_length_options_);
    objc_sel!(newRenderPipelineStateWithDescriptor_error_);
    objc_sel!(newLibraryWithURL_error_);
    objc_sel!(newLibraryWithSource_options_error_);
    objc_sel!(newTextureWithDescriptor_);

    // MTLCommandQueue
    objc_sel!(commandBuffer);

    // MTLCommandBuffer
    objc_sel!(renderCommandEncoderWithDescriptor_);
    objc_sel!(presentDrawable_);
    objc_sel!(commit);
    objc_sel!(waitUntilCompleted);

    // MTLRenderCommandEncoder
    objc_sel!(endEncoding);
    objc_sel!(setRenderPipelineState_);
    objc_sel!(setVertexBytes_length_atIndex_);
    objc_sel!(setVertexBuffer_offset_atIndex_);
    objc_sel!(drawPrimitives_vertexStart_vertexCount_instanceCount_);
    objc_sel!(setFragmentTexture_atIndex_);
    objc_sel!(setFragmentBuffer_offset_atIndex_);
    objc_sel!(setFragmentBytes_length_atIndex_);

    //MTLBuffer
    objc_sel!(contents);
    objc_sel!(length);

    // MTLRenderPipelineDescriptor
    objc_prop_sel!(vertexFunction);
    objc_prop_sel!(fragmentFunction);
    objc_prop_sel!(colorAttachments);

    // MTLLibrary
    objc_sel!(newFunctionWithName_);

    // MTLRenderPipelineColorAttachmentDescriptorArray
    objc_sel!(objectAtIndexedSubscript_);

    // MTLRenderPipelineColorAttachmentDescriptor
    objc_prop_sel!(pixelFormat);
    objc_sel!(isBlendingEnabled);
    objc_sel!(setBlendingEnabled_);
    objc_prop_sel!(sourceRGBBlendFactor);
    objc_prop_sel!(sourceAlphaBlendFactor);
    objc_prop_sel!(destinationRGBBlendFactor);
    objc_prop_sel!(destinationAlphaBlendFactor);

    // NSApplicationDelegate
    objc_sel!(applicationShouldTerminate_);

    // NSWindowDelegate
    objc_sel!(windowWillResize_toSize_);
    objc_sel!(windowDidEndLiveResize_);
    objc_sel!(windowShouldClose_);

    // NSEvent
    objc_prop_sel!(characters);
    objc_prop_sel!(charactersIgnoringModifiers);
    objc_prop_sel!(keyCode);
    objc_prop_sel!(modifierFlags);
    objc_prop_sel!(timestamp);

    // NSResponder
    objc_prop_sel!(acceptsFirstResponder);
    objc_sel!(flagsChanged_);
    objc_sel!(keyDown_);
    objc_sel!(keyUp_);

    // NSMenu
    objc_sel!(initWithTitle_);
    objc_sel!(addItem_);
    objc_sel!(itemAtIndex_);
    objc_sel!(insertItemWithTitle_action_keyEquivalent_atIndex_);

    // NSMenuItem
    objc_sel!(initWithTitle_action_keyEquivalent_);
    objc_prop_sel!(submenu);

    // NSProcessInfo
    objc_prop_sel!(processInfo);
    objc_prop_sel!(systemUptime);

    // MTLVertexDescriptor
    objc_prop_sel!(vertexDescriptor);
    objc_prop_sel!(attributes);
    objc_prop_sel!(layouts);

    // MTLVertexAttributeDescriptor
    objc_prop_sel!(format);
    objc_prop_sel!(offset);
    objc_prop_sel!(bufferIndex);

    // MTLVertexBufferLayoutDescriptor
    objc_prop_sel!(stride);

    // MTLTextureDescriptor
    objc_sel!(texture2DDescriptorWithPixelFormat_width_height_mipmapped_);
    objc_prop_sel!(width);

    // MTLTexture
    objc_sel!(replaceRegion_mipmapLevel_withBytes_bytesPerRow_);
    objc_prop_sel!(textureType);
}

pub fn init_objc() {
    init_objc_core();

    NSUrl::init();
    NSError::init();
    NSApplication::init();
    NSWindow::init();
    MTKView::init();
    MTLRenderPassDescriptor::init();
    CAMetalDrawable::init();
    MTLRenderPipelineDescriptor::init();
    MTLRenderPipelineColorAttachmentDescriptorArray::init();
    MTLRenderPipelineColorAttachmentDescriptor::init();
    MTLCompileOptions::init();
    NSEvent::init();
    NSMenu::init();
    NSMenuItem::init();
    NSProcessInfo::init();
    MTLVertexDescriptor::init();
    MTLVertexAttributeDescriptor::init();
    MTLVertexAttributeDescriptorArray::init();
    MTLVertexBufferLayoutDescriptor::init();
    MTLVertexBufferLayoutDescriptorArray::init();
    MTLTextureDescriptor::init();

    // NSError
    objc_prop_sel_init!(code);
    objc_prop_sel_init!(domain);
    objc_prop_sel_init!(localizedDescription);

    // NSUrl
    sel::URLWithString_.init();

    // NSApplication
    sel::sharedApplication.init();
    sel::setActivationPolicy_.init();
    sel::run.init();
    sel::stop_.init();
    sel::terminate_.init();
    objc_prop_sel_init!(mainMenu);
    objc_prop_sel_init!(isRunning);

    // NSWindow
    sel::initWithContentRect_styleMask_backing_defer_.init();
    sel::makeMainWindow.init();
    sel::center.init();
    sel::setContentSize_.init();
    objc_prop_sel_init!(title);
    objc_prop_sel_init!(isVisible);
    objc_prop_sel_init!(contentView);
    objc_prop_sel_init!(contentAspectRatio);
    objc_prop_sel_init!(contentResizeIncrements);
    objc_prop_sel_init!(contentLayoutRect);
    objc_prop_sel_init!(frame);
    objc_prop_sel_init!(contentMinSize);

    // MTKView
    sel::initWithFrame_device_.init();
    sel::drawRect_.init();
    objc_prop_sel_init!(delegate);
    objc_prop_sel_init!(clearColor);
    objc_prop_sel_init!(currentRenderPassDescriptor);
    objc_prop_sel_init!(device);
    objc_prop_sel_init!(currentDrawable);
    objc_prop_sel_init!(colorPixelFormat);
    objc_prop_sel_init!(preferredFramesPerSecond);

    // MTKViewDelegate
    sel::drawInMTKView_.init();
    sel::mtkView_drawableSizeWillChange_.init();

    // MTLDevice
    sel::newCommandQueue.init();
    sel::newBufferWithBytes_length_options_.init();
    sel::newRenderPipelineStateWithDescriptor_error_.init();
    sel::newLibraryWithURL_error_.init();
    sel::newLibraryWithSource_options_error_.init();
    sel::newTextureWithDescriptor_.init();

    // MTLCommandQueue
    sel::commandBuffer.init();

    // MTLCommandBuffer
    sel::renderCommandEncoderWithDescriptor_.init();
    sel::presentDrawable_.init();
    sel::commit.init();
    sel::waitUntilCompleted.init();

    // MTLRenderCommandEncoder
    sel::endEncoding.init();
    sel::setRenderPipelineState_.init();
    sel::setVertexBytes_length_atIndex_.init();
    sel::setVertexBuffer_offset_atIndex_.init();
    sel::drawPrimitives_vertexStart_vertexCount_instanceCount_.init();
    sel::setFragmentBytes_length_atIndex_.init();
    sel::setFragmentBuffer_offset_atIndex_.init();
    sel::setFragmentTexture_atIndex_.init();

    // MTLBuffer
    sel::contents.init();
    sel::length.init();

    // MTLRenderPipelineDescriptor
    objc_prop_sel_init!(vertexFunction);
    objc_prop_sel_init!(fragmentFunction);
    objc_prop_sel_init!(colorAttachments);

    // MTLLibrary
    sel::newFunctionWithName_.init();

    // MTLRenderPipelineColorAttachmentDescriptorArray
    sel::objectAtIndexedSubscript_.init();

    // MTLRenderPipelineColorAttachmentDescriptor
    objc_prop_sel_init!(pixelFormat);
    sel::isBlendingEnabled.init();
    sel::setBlendingEnabled_.init();
    objc_prop_sel_init!(sourceRGBBlendFactor);
    objc_prop_sel_init!(sourceAlphaBlendFactor);
    objc_prop_sel_init!(destinationRGBBlendFactor);
    objc_prop_sel_init!(destinationAlphaBlendFactor);

    // NSApplicationDelegate
    sel::applicationShouldTerminate_.init();

    // NSWindowDelegate
    sel::windowWillResize_toSize_.init();
    sel::windowDidEndLiveResize_.init();
    sel::windowShouldClose_.init();
    sel::keyDown_.init();
    sel::keyUp_.init();

    // NSEvent
    objc_prop_sel_init!(characters);
    objc_prop_sel_init!(charactersIgnoringModifiers);
    objc_prop_sel_init!(keyCode);
    objc_prop_sel_init!(modifierFlags);
    objc_prop_sel_init!(timestamp);

    // NSResponder
    objc_prop_sel_init!(acceptsFirstResponder);
    sel::flagsChanged_.init();

    // NSMenu
    sel::initWithTitle_.init();
    sel::addItem_.init();
    sel::itemAtIndex_.init();
    sel::insertItemWithTitle_action_keyEquivalent_atIndex_.init();

    // NSMenuItem
    sel::initWithTitle_action_keyEquivalent_.init();
    objc_prop_sel_init!(submenu);

    // NSProcessInfo
    objc_prop_sel_init!(processInfo);
    objc_prop_sel_init!(systemUptime);

    // MTLVertexDescriptor
    objc_prop_sel_init!(vertexDescriptor);
    objc_prop_sel_init!(attributes);
    objc_prop_sel_init!(layouts);

    // MTLVertexAttributeDescriptor
    objc_prop_sel_init!(format);
    objc_prop_sel_init!(offset);
    objc_prop_sel_init!(bufferIndex);

    // MTLVertexBufferLayoutDescriptor
    objc_prop_sel_init!(stride);

    // MTLTextureDescriptor
    sel::texture2DDescriptorWithPixelFormat_width_height_mipmapped_.init();
    objc_prop_sel_init!(width);

    // MTLTexture
    sel::replaceRegion_mipmapLevel_withBytes_bytesPerRow_.init();
    objc_prop_sel_init!(textureType);
}

impl NSError::IPtr {
    objc_prop_impl!(code, NSInteger, code);
    objc_prop_impl!(domain, NSString::IPtr, domain);
    objc_prop_impl!(localizedDescription, NSString::IPtr, localized_desc);
}

impl Display for NSError::IPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "NSError(code = {}, domain = {}, description = {})",
            self.code(),
            self.domain(),
            self.localized_desc()
        )
    }
}

impl Error for NSError::IPtr {}

impl NSApplication::IPtr {
    pub fn shared() -> NSApplication::IPtr {
        unsafe { msg0::<NSApplication::IPtr>(NSApplication::obj(), sel::sharedApplication.sel()) }
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

    pub fn stop(&self, sender: OPtr) {
        unsafe { msg1::<(), OPtr>(self.0, sel::stop_.sel(), sender) };
    }

    pub fn terminate(&self, sender: OPtr) {
        unsafe { msg1::<(), OPtr>(self.0, sel::terminate_.sel(), sender) };
    }

    objc_prop_impl!(mainMenu, Option<NSMenu::IPtr>, main_menu, set_main_menu);
    objc_prop_impl!(isRunning, bool, running);
    objc_prop_impl!(
        delegate,
        NSApplicationDelegate::PPtr,
        delegate,
        set_delegate
    );
}

#[repr(i64)]
pub enum NSApplicationActivationPolicy {
    Regular = 0,
}

#[repr(i64)]
pub enum NSBackingStoreType {
    Buffered = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct NSTimeInterval(f64);

impl Add<f64> for NSTimeInterval {
    type Output = Self;

    fn add(self, other: f64) -> Self {
        Self(self.0 + other)
    }
}

impl Sub for NSTimeInterval {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl NSTimeInterval {
    pub fn to_secs(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NSWindowStyleMask(NSUInteger);

derive_BitOr!(NSWindowStyleMask);

impl NSWindowStyleMask {
    pub const TITLED: Self = Self(1);
    pub const CLOSABLE: Self = Self(2);
    pub const MINIATURIZABLE: Self = Self(4);
    pub const RESIZABLE: Self = Self(8);
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

impl NSWindow::IPtr {
    pub fn init(
        alloc_obj: AllocObj<NSWindow::IPtr>,
        rect: CGRect,
        style_mask: NSWindowStyleMask,
        backing: NSBackingStoreType,
        defer: bool,
    ) -> NSWindow::IPtr {
        unsafe {
            msg4::<NSWindow::IPtr, CGRect, NSWindowStyleMask, NSBackingStoreType, Bool>(
                alloc_obj.obj(),
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

    pub fn set_content_size(&self, size: CGSize) {
        unsafe { msg1::<(), CGSize>(self.0, sel::setContentSize_.sel(), size) };
    }

    pub fn center(&self) {
        unsafe { msg0::<()>(self.0, sel::center.sel()) };
    }

    objc_prop_impl!(acceptsFirstResponder, Bool, accepts_first_responder);
    objc_prop_impl!(delegate, NSWindowDelegate::PPtr, delegate, set_delegate);
    objc_prop_impl!(title, NSString::IPtr, title, set_title);
    objc_prop_impl!(isVisible, bool, is_visible, set_is_visible);
    objc_prop_impl!(contentView, MTKView::IPtr, content_view, set_content_view);
    objc_prop_impl!(
        contentAspectRatio,
        CGSize,
        content_aspect_ratio,
        set_content_aspect_ratio
    );
    objc_prop_impl!(
        contentResizeIncrements,
        CGSize,
        content_resize_increments,
        set_content_resize_increments
    );
    objc_prop_impl!(contentLayoutRect, CGRect, content_rect);
    objc_prop_impl!(frame, CGRect, frame, set_frame);
    objc_prop_impl!(
        contentMinSize,
        CGSize,
        content_min_size,
        set_content_min_size
    );
}

#[repr(C)]
#[derive(Debug)]
pub struct MTLClearColor {
    red: f64,
    green: f64,
    blue: f64,
    alpha: f64,
}

#[repr(i64)]
#[derive(Debug)]
pub enum MTLPrimitiveType {
    Point = 0,
    Line = 1,
    LineStrip = 2,
    Triangle = 3,
    TriangleStrip = 4,
}

#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub enum MTLPixelFormat {
    R8Uint = 13,
    RGBA8Unorm = 70,
    RGBA16Unorm = 110,
}

impl MTKView::IPtr {
    pub fn override_draw_rect(f: extern "C" fn(MTKView::IPtr, Sel, CGRect)) -> bool {
        unsafe {
            MTKView::cls().add_method1(
                sel::drawRect_.sel(),
                f,
                c"v@:{CGRect={CGPoint=dd}{CGSize=dd}}",
            )
        }
    }

    objc_prop_impl!(delegate, MTKViewDelegate::PPtr, delegate, set_delegate);
    objc_prop_impl!(clearColor, MTLClearColor, clear_color, set_clear_color);

    objc_prop_impl!(
        currentRenderPassDescriptor,
        Option<MTLRenderPassDescriptor::IPtr>,
        current_rendpass_desc,
        set_current_rendpass_desc
    );

    objc_prop_impl!(
        currentDrawable,
        Option<CAMetalDrawable::IPtr>,
        current_drawable,
        set_current_drawable
    );

    objc_prop_impl!(device, Option<MTLDevice::PPtr>, device, set_device);
    objc_prop_impl!(
        colorPixelFormat,
        MTLPixelFormat,
        color_pixel_fmt,
        set_color_pixel_fmt
    );

    objc_prop_impl!(
        preferredFramesPerSecond,
        NSInteger,
        preferred_fps,
        set_preferred_fps
    );

    pub fn init(
        alloc_obj: AllocObj<MTKView::IPtr>,
        frame: CGRect,
        device: MTLDevice::PPtr,
    ) -> Self {
        unsafe {
            msg2::<MTKView::IPtr, CGRect, MTLDevice::PPtr>(
                alloc_obj.obj(),
                sel::initWithFrame_device_.sel(),
                frame,
                device,
            )
        }
    }
}

impl MTLDevice::PPtr {
    pub fn get_default() -> MTLDevice::PPtr {
        let ptr = MTLCreateSystemDefaultDevice();
        Self(OPtr::new(ptr))
    }

    pub fn new_cmd_queue(&self) -> Option<MTLCommandQueue::PPtr> {
        unsafe { msg0::<Option<MTLCommandQueue::PPtr>>(self.0, sel::newCommandQueue.sel()) }
    }

    pub fn new_buf<T: AsBytes + ?Sized>(
        &self,
        buf: &T,
        options: MTLResourceOptions,
    ) -> Option<MTLBuffer::PPtr> {
        let bytes = buf.as_bytes();
        unsafe {
            msg3::<Option<MTLBuffer::PPtr>, *const u8, NSUInteger, MTLResourceOptions>(
                self.0,
                sel::newBufferWithBytes_length_options_.sel(),
                bytes.as_ptr(),
                size_of_val(bytes) as NSUInteger,
                options,
            )
        }
    }

    pub fn new_tex(&self, desc: MTLTextureDescriptor::IPtr) -> Option<MTLTexture::PPtr> {
        unsafe {
            msg1::<Option<MTLTexture::PPtr>, MTLTextureDescriptor::IPtr>(
                self.0,
                sel::newTextureWithDescriptor_.sel(),
                desc,
            )
        }
    }

    pub fn new_rend_pl_state(
        &self,
        desc: MTLRenderPipelineDescriptor::IPtr,
    ) -> Result<MTLRenderPipelineState::PPtr, NSError::IPtr> {
        let mut error = None;
        let res = unsafe {
            msg2::<
                Option<MTLRenderPipelineState::PPtr>,
                MTLRenderPipelineDescriptor::IPtr,
                &mut Option<NSError::IPtr>,
            >(
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

    pub fn new_lib_with_url(&self, url: NSUrl::IPtr) -> Result<MTLLibrary::PPtr, NSError::IPtr> {
        let mut error = None;
        let res = unsafe {
            msg2::<Option<MTLLibrary::PPtr>, NSUrl::IPtr, &mut Option<NSError::IPtr>>(
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

    pub fn new_lib_from_source(
        &self,
        source: NSString::IPtr,
        options: MTLCompileOptions::IPtr,
    ) -> Result<MTLLibrary::PPtr, NSError::IPtr> {
        let mut error = None;
        let res = unsafe {
            msg3::<
                Option<MTLLibrary::PPtr>,
                NSString::IPtr,
                MTLCompileOptions::IPtr,
                &mut Option<NSError::IPtr>,
            >(
                self.0,
                sel::newLibraryWithSource_options_error_.sel(),
                source,
                options,
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

impl MTLCommandQueue::PPtr {
    pub fn cmd_buf(&self) -> Option<MTLCommandBuffer::PPtr> {
        unsafe { msg0::<Option<MTLCommandBuffer::PPtr>>(self.0, sel::commandBuffer.sel()) }
    }
}

impl MTLCommandBuffer::PPtr {
    pub fn rencoder_with_desc(
        &self,
        pass_desc: MTLRenderPassDescriptor::IPtr,
    ) -> Option<MTLRenderCommandEncoder::PPtr> {
        unsafe {
            msg1::<Option<MTLRenderCommandEncoder::PPtr>, MTLRenderPassDescriptor::IPtr>(
                self.0,
                sel::renderCommandEncoderWithDescriptor_.sel(),
                pass_desc,
            )
        }
    }

    pub fn present_drawable(&self, drawable: CAMetalDrawable::IPtr) {
        unsafe { msg1::<(), CAMetalDrawable::IPtr>(self.0, sel::presentDrawable_.sel(), drawable) }
    }

    pub fn commit(&self) {
        unsafe { msg0::<()>(self.0, sel::commit.sel()) }
    }

    pub fn wait_completion(&self) {
        unsafe { msg0::<()>(self.0, sel::waitUntilCompleted.sel()) }
    }
}

impl MTLRenderCommandEncoder::PPtr {
    pub fn set_rend_pl_state(&self, state: MTLRenderPipelineState::PPtr) {
        unsafe {
            msg1::<(), MTLRenderPipelineState::PPtr>(
                self.0,
                sel::setRenderPipelineState_.sel(),
                state,
            )
        }
    }

    pub fn set_vtex_bytes<T: AsBytes + ?Sized>(&self, bytes: &T, index: usize) {
        let bytes = bytes.as_bytes();
        unsafe {
            msg3::<(), *const u8, NSUInteger, NSUInteger>(
                self.0,
                sel::setVertexBytes_length_atIndex_.sel(),
                bytes.as_ptr(),
                size_of_val(bytes) as NSUInteger,
                index as NSUInteger,
            )
        }
    }

    pub fn set_vtex_buf(&self, buf: MTLBuffer::PPtr, offset: usize, index: usize) {
        unsafe {
            msg3::<(), MTLBuffer::PPtr, NSUInteger, NSUInteger>(
                self.0,
                sel::setVertexBuffer_offset_atIndex_.sel(),
                buf,
                offset as NSUInteger,
                index as NSUInteger,
            )
        }
    }

    pub fn set_frag_tex(&self, tex: MTLTexture::PPtr, index: usize) {
        unsafe {
            msg2::<(), MTLTexture::PPtr, NSUInteger>(
                self.0,
                sel::setFragmentTexture_atIndex_.sel(),
                tex,
                index as NSUInteger,
            )
        }
    }

    pub fn set_frag_buf(&self, buf: MTLBuffer::PPtr, offset: usize, index: usize) {
        unsafe {
            msg3::<(), MTLBuffer::PPtr, NSUInteger, NSUInteger>(
                self.0,
                sel::setFragmentBuffer_offset_atIndex_.sel(),
                buf,
                offset as NSUInteger,
                index as NSUInteger,
            )
        }
    }
    pub fn set_frag_bytes<T: AsBytes + ?Sized>(&self, bytes: &T, index: usize) {
        let bytes = bytes.as_bytes();
        unsafe {
            msg3::<(), *const u8, NSUInteger, NSUInteger>(
                self.0,
                sel::setFragmentBytes_length_atIndex_.sel(),
                bytes.as_ptr(),
                size_of_val(bytes) as NSUInteger,
                index as NSUInteger,
            )
        }
    }
    pub fn draw_primitives(
        &self,
        primitive_type: MTLPrimitiveType,
        vertex_start: usize,
        vertex_count: usize,
        instance_count: usize,
    ) {
        unsafe {
            msg4::<(), MTLPrimitiveType, NSUInteger, NSUInteger, NSUInteger>(
                self.0,
                sel::drawPrimitives_vertexStart_vertexCount_instanceCount_.sel(),
                primitive_type,
                vertex_start as NSUInteger,
                vertex_count as NSUInteger,
                instance_count as NSUInteger,
            )
        }
    }

    pub fn end(&self) {
        unsafe { msg0::<()>(self.0, sel::endEncoding.sel()) }
    }
}

#[repr(transparent)]
pub struct MTLResourceOptions(NSUInteger);

derive_BitOr!(MTLResourceOptions);

impl MTLResourceOptions {
    pub const DEFAULT: Self = Self(0);
}

impl MTLRenderPipelineDescriptor::IPtr {
    pub fn new() -> Self {
        let alloc_obj = MTLRenderPipelineDescriptor::alloc();
        unsafe { msg0::<MTLRenderPipelineDescriptor::IPtr>(alloc_obj.obj(), init.sel()) }
    }

    objc_prop_impl!(vertexFunction, MTLFunction::PPtr, vtex_fn, set_vtex_fn);
    objc_prop_impl!(fragmentFunction, MTLFunction::PPtr, frag_fn, set_frag_fn);
    objc_prop_impl!(
        colorAttachments,
        MTLRenderPipelineColorAttachmentDescriptorArray::IPtr,
        color_attach,
        set_color_attach
    );
    objc_prop_impl!(
        vertexDescriptor,
        MTLVertexDescriptor::IPtr,
        vtex_desc,
        set_vtex_desc
    );
}

impl NSUrl::IPtr {
    pub fn new(s: &CStr) -> NSUrl::IPtr {
        // SAFETY: OK.
        unsafe {
            msg1::<NSUrl::IPtr, NSString::IPtr>(
                NSUrl::obj(),
                sel::URLWithString_.sel(),
                NSString::IPtr::new(s),
            )
        }
    }
}

impl MTLLibrary::PPtr {
    pub fn new_fn(&self, name: &CStr) -> MTLFunction::PPtr {
        // SAFETY: OK.
        unsafe {
            msg1::<MTLFunction::PPtr, NSString::IPtr>(
                self.0,
                sel::newFunctionWithName_.sel(),
                NSString::IPtr::new(name),
            )
        }
    }
}

impl MTLRenderPipelineColorAttachmentDescriptorArray::IPtr {
    pub fn at(&self, index: usize) -> MTLRenderPipelineColorAttachmentDescriptor::IPtr {
        // SAFETY: OK.
        unsafe {
            msg1::<MTLRenderPipelineColorAttachmentDescriptor::IPtr, NSUInteger>(
                self.0,
                sel::objectAtIndexedSubscript_.sel(),
                index as NSUInteger,
            )
        }
    }
}

impl MTLRenderPipelineColorAttachmentDescriptor::IPtr {
    objc_prop_impl!(clearColor, MTLClearColor, clear_color, set_clear_color);
    objc_prop_impl!(pixelFormat, MTLPixelFormat, pixel_fmt, set_pixel_fmt);

    pub fn blend_enabled(&self) -> bool {
        unsafe { msg0::<Bool>(self.0, sel::isBlendingEnabled.sel()) }
    }

    pub fn set_blend_enabled(&self, enabled: bool) {
        unsafe {
            msg1::<(), Bool>(self.0, sel::setBlendingEnabled_.sel(), enabled);
        }
    }

    objc_prop_impl!(
        destinationAlphaBlendFactor,
        MTLBlendFactor,
        dest_alpha_blend_factor,
        set_dest_alpha_blend_factor
    );
    objc_prop_impl!(
        destinationRGBBlendFactor,
        MTLBlendFactor,
        dest_rgb_blend_factor,
        set_dest_rgb_blend_factor
    );
    objc_prop_impl!(
        sourceAlphaBlendFactor,
        MTLBlendFactor,
        source_alpha_blend_factor,
        set_source_alpha_blend_factor
    );
    objc_prop_impl!(
        sourceRGBBlendFactor,
        MTLBlendFactor,
        source_rgb_blend_factor,
        set_source_rgb_blend_factor
    );
}

impl MTLRenderPassDescriptor::IPtr {
    objc_prop_impl!(
        colorAttachments,
        MTLRenderPipelineColorAttachmentDescriptorArray::IPtr,
        color_attach,
        set_color_attach
    );
}

impl MTLClearColor {
    pub fn new(red: f64, green: f64, blue: f64, alpha: f64) -> MTLClearColor {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl MTLCompileOptions::IPtr {
    pub fn new() -> Self {
        let alloc_obj = MTLCompileOptions::alloc();
        unsafe { msg0::<MTLCompileOptions::IPtr>(alloc_obj.obj(), init.sel()) }
    }
}

impl MTKViewDelegate::PPtr {
    pub fn implement<T>(
        cls: &TypedCls<T, Self>,
        draw: extern "C" fn(TypedObj<T>, Sel, MTKView::IPtr),
        size_change: extern "C" fn(TypedObj<T>, Sel, MTKView::IPtr, CGSize),
    ) -> bool {
        unsafe {
            cls.cls()
                .add_method1(sel::drawInMTKView_.sel(), draw, c"v@:@");
            cls.cls().add_method2(
                sel::mtkView_drawableSizeWillChange_.sel(),
                size_change,
                c"v@:@{CGSize=dd}",
            );
        }
        cls.cls().add_protocol(c"MTKViewDelegate")
    }
}

unsafe impl Protocol for NSWindowDelegate::PPtr {
    fn new(obj: OPtr) -> Self {
        Self(obj)
    }
}

impl NSWindowDelegate::PPtr {
    pub fn implement<T>(
        cls: &TypedCls<T, Self>,
        should_close: extern "C" fn(TypedObj<T>, Sel, OPtr) -> bool,
        will_resize: extern "C" fn(TypedObj<T>, Sel, OPtr),
    ) -> bool {
        unsafe {
            cls.cls()
                .add_method1(sel::windowShouldClose_.sel(), should_close, c"c@:@");
            cls.cls()
                .add_method1(sel::windowDidEndLiveResize_.sel(), will_resize, c"v@:@");
        }
        cls.cls().add_protocol(c"NSWindowDelegate")
    }
}

#[derive(Debug)]
#[repr(u64)]
pub enum NSApplicationTerminateReply {
    NSTerminateCancel = 0,
    NSTerminateNow = 1,
    NSTerminateLater = 2,
}

unsafe impl Protocol for NSApplicationDelegate::PPtr {
    fn new(obj: OPtr) -> Self {
        Self(obj)
    }
}

impl NSApplicationDelegate::PPtr {
    pub fn implement<T>(
        cls: &TypedCls<T, Self>,
        should_terminate: extern "C" fn(TypedObj<T>, Sel, OPtr) -> NSApplicationTerminateReply,
    ) -> bool {
        unsafe {
            cls.cls().add_method1(
                sel::applicationShouldTerminate_.sel(),
                should_terminate,
                c"c@:@",
            );
        }
        cls.cls().add_protocol(c"NSApplicationDelegate")
    }
}

impl NSEvent::IPtr {
    objc_prop_impl!(characters, NSString::IPtr, chars);
    objc_prop_impl!(
        charactersIgnoringModifiers,
        NSString::IPtr,
        chars_ignore_mod
    );
    objc_prop_impl!(keyCode, u16, key_code);
    objc_prop_impl!(modifierFlags, NSEventModifierFlags, mod_flags);
    objc_prop_impl!(timestamp, NSTimeInterval, timestamp);
}

pub struct NSResponder;

impl NSResponder {
    pub fn override_key_down<T>(
        cls: CPtr,
        key_down: extern "C" fn(TypedObj<T>, Sel, NSEvent::IPtr),
    ) {
        unsafe {
            cls.add_method1(sel::keyDown_.sel(), key_down, c"v@:@");
        }
    }

    pub fn override_key_up<T>(cls: CPtr, key_up: extern "C" fn(TypedObj<T>, Sel, NSEvent::IPtr)) {
        unsafe {
            cls.add_method1(sel::keyUp_.sel(), key_up, c"v@:@");
        }
    }

    pub fn override_flag_changed<T>(
        cls: CPtr,
        key_down: extern "C" fn(TypedObj<T>, Sel, NSEvent::IPtr),
    ) {
        unsafe {
            cls.add_method1(sel::flagsChanged_.sel(), key_down, c"v@:@");
        }
    }

    extern "C" fn yes(_slf: TypedObj<()>, _sel: Sel) -> bool {
        true
    }

    pub fn override_accepts_first_responder_as_true(cls: CPtr) {
        unsafe {
            cls.add_method0(sel::acceptsFirstResponder::GETTER.sel(), Self::yes, c"c@:");
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct NSEventModifierFlags(NSUInteger);

derive_BitOr!(NSEventModifierFlags);

impl NSEventModifierFlags {
    const CAPSLOCK: Self = Self(1 << 16);
    const SHIFT: Self = Self(1 << 17);
    const CTRL: Self = Self(1 << 18);
    const OPTION: Self = Self(1 << 19);
    const CMD: Self = Self(1 << 20);
    const NUMPAD: Self = Self(1 << 21);
    const HELP: Self = Self(1 << 22);
    const FUNCTION: Self = Self(1 << 23);
}

impl NSMenu::IPtr {
    pub fn new(title: &CStr) -> Self {
        let alloc_obj = NSMenu::alloc();
        unsafe {
            msg1::<Self, NSString::IPtr>(
                alloc_obj.obj(),
                sel::initWithTitle_.sel(),
                NSString::IPtr::new(title),
            )
        }
    }

    pub fn add_item(self, item: NSMenuItem::IPtr) {
        unsafe {
            msg1::<(), NSMenuItem::IPtr>(self.0, sel::addItem_.sel(), item);
        }
    }

    pub fn item_at(self, idx: usize) -> NSMenuItem::IPtr {
        unsafe {
            msg1::<NSMenuItem::IPtr, NSInteger>(self.0, sel::itemAtIndex_.sel(), idx as NSInteger)
        }
    }

    pub fn insert_item(
        self,
        title: &CStr,
        action: Option<Sel>,
        key_equiv: &CStr,
        idx: usize,
    ) -> NSMenuItem::IPtr {
        unsafe {
            msg4::<NSMenuItem::IPtr, NSString::IPtr, Option<Sel>, NSString::IPtr, NSInteger>(
                self.0,
                sel::insertItemWithTitle_action_keyEquivalent_atIndex_.sel(),
                NSString::IPtr::new(title),
                action,
                NSString::IPtr::new(key_equiv),
                idx as NSInteger,
            )
        }
    }
}

impl NSMenuItem::IPtr {
    pub fn new(title: &CStr, action: Option<Sel>, key_equiv: &CStr) -> Self {
        let alloc_obj = NSMenuItem::alloc();
        unsafe {
            msg3::<Self, NSString::IPtr, Option<Sel>, NSString::IPtr>(
                alloc_obj.obj(),
                sel::initWithTitle_action_keyEquivalent_.sel(),
                NSString::IPtr::new(title),
                action,
                NSString::IPtr::new(key_equiv),
            )
        }
    }

    objc_prop_impl!(title, NSString::IPtr, title, set_title);
    objc_prop_impl!(submenu, NSMenu::IPtr, submenu, set_submenu);
}

impl NSProcessInfo::IPtr {
    pub fn process_info() -> Self {
        unsafe {
            msg0::<NSProcessInfo::IPtr>(NSProcessInfo::cls().obj(), sel::processInfo::GETTER.sel())
        }
    }

    objc_prop_impl!(systemUptime, NSTimeInterval, system_uptime);
}

impl MTLVertexDescriptor::IPtr {
    pub fn new() -> Self {
        unsafe {
            msg0::<Self>(
                MTLVertexDescriptor::CLS.obj(),
                sel::vertexDescriptor::GETTER.sel(),
            )
        }
    }

    objc_prop_impl!(
        attributes,
        MTLVertexAttributeDescriptorArray::IPtr,
        attributes
    );
    objc_prop_impl!(layouts, MTLVertexBufferLayoutDescriptorArray::IPtr, layouts);
}

impl MTLVertexAttributeDescriptorArray::IPtr {
    pub fn at(&self, idx: usize) -> MTLVertexAttributeDescriptor::IPtr {
        unsafe {
            msg1::<MTLVertexAttributeDescriptor::IPtr, NSUInteger>(
                self.obj(),
                sel::objectAtIndexedSubscript_.sel(),
                idx as NSUInteger,
            )
        }
    }
}

impl MTLVertexBufferLayoutDescriptorArray::IPtr {
    pub fn at(&self, idx: usize) -> MTLVertexBufferLayoutDescriptor::IPtr {
        unsafe {
            msg1::<MTLVertexBufferLayoutDescriptor::IPtr, NSUInteger>(
                self.obj(),
                sel::objectAtIndexedSubscript_.sel(),
                idx as NSUInteger,
            )
        }
    }
}

impl MTLVertexAttributeDescriptor::IPtr {
    objc_prop_impl!(format, MTLVertexFormat, format, set_format);
    objc_prop_impl!(offset, NSUInteger, offset, set_offset);
    objc_prop_impl!(bufferIndex, NSUInteger, buf_idx, set_buffer_index);
}

impl MTLVertexBufferLayoutDescriptor::IPtr {
    objc_prop_impl!(stride, NSUInteger, stride, set_stride);
}

#[repr(i64)]
#[derive(Debug)]
pub enum MTLVertexFormat {
    Invalid = 0,
    Float = 28,
    Float2 = 29,
    Float3 = 30,
    Float4 = 31,
}

impl MTLBuffer::PPtr {
    pub fn len(&self) -> usize {
        unsafe { msg0::<NSUInteger>(self.obj(), sel::length.sel()) as usize }
    }

    pub fn contents(&self) -> &[u8] {
        let len = self.len();
        let ptr = unsafe { msg0::<*mut c_void>(self.obj(), sel::contents.sel()) };
        unsafe { std::slice::from_raw_parts(ptr as *const u8, len) }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct MTLOrigin {
    pub x: NSUInteger,
    pub y: NSUInteger,
    pub z: NSUInteger,
}

#[derive(Debug)]
#[repr(C)]
pub struct MTLSize {
    pub width: NSUInteger,
    pub height: NSUInteger,
    pub depth: NSUInteger,
}

#[derive(Debug)]
#[repr(C)]
pub struct MTLRegion {
    pub origin: MTLOrigin,
    pub size: MTLSize,
}

impl MTLTextureDescriptor::IPtr {
    pub fn new() -> Self {
        Self::alloc_init()
    }

    pub fn new_2d(pixel_fmt: MTLPixelFormat, width: usize, height: usize) -> Self {
        unsafe {
            msg4::<MTLTextureDescriptor::IPtr, MTLPixelFormat, NSUInteger, NSUInteger, Bool>(
                Self::cls().obj(),
                sel::texture2DDescriptorWithPixelFormat_width_height_mipmapped_.sel(),
                pixel_fmt,
                width as NSUInteger,
                height as NSUInteger,
                false,
            )
        }
    }

    objc_prop_impl!(textureType, MTLTextureType, texture_type, set_texture_type);
    objc_prop_impl!(pixelFormat, MTLPixelFormat, pixel_format, set_pixel_format);
    objc_prop_impl!(width, NSUInteger, width, set_width);
}

impl MTLTexture::PPtr {
    pub fn replace<T: AsBytes + ?Sized>(&self, region: MTLRegion, bytes: &T, per_row: usize) {
        let bytes = bytes.as_bytes();
        unsafe {
            msg4::<(), MTLRegion, NSUInteger, *const u8, NSUInteger>(
                self.obj(),
                sel::replaceRegion_mipmapLevel_withBytes_bytesPerRow_.sel(),
                region,
                0,
                bytes.as_ptr(),
                per_row as NSUInteger,
            )
        }
    }

    objc_prop_impl!(textureType, MTLTextureType, texture_type, set_texture_type);
}

#[repr(i64)]
#[derive(Debug)]
pub enum MTLTextureUsage {
    ShaderRead = 1,
}

#[derive(Debug)]
#[repr(u64)]
pub enum MTLTextureType {
    T1D = 0,
    T2D = 2,
}

#[derive(Debug)]
#[repr(u64)]
pub enum MTLBlendFactor {
    Zero = 0,
    One = 1,
    SourceAlpha = 4,
    OneMinusSourceAlpha = 5,
}
