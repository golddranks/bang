// We only support 64-bit macOS on Apple Silicon
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
mod crimes;
mod wrappers;

pub use wrappers::{
    CGPoint, CGRect, CGSize, MTKView, MTLBuffer, MTLClearColor, MTLCommandQueue, MTLCompileOptions,
    MTLDevice, MTLPixelFormat, MTLPrimitiveType, MTLRenderPipelineDescriptor,
    MTLRenderPipelineState, MTLResourceOptions, NSApplication, NSApplicationActivationPolicy,
    NSBackingStoreType, NSString, NSUrl, NSWindow, NSWindowStyleMask, TypedMTKViewDelegate,
    TypedMTKViewDelegateCls, cls, init_objc,
};

pub use crimes::{AllocObj, InstancePtr, Obj, Sel, TypedPtr};
