// We only support 64-bit macOS on Apple Silicon
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
mod crimes;
mod wrappers;

pub use wrappers::{
    CGPoint, CGRect, CGSize, MTKView, MTLBuffer, MTLCommandQueue, MTLDevice,
    MTLRenderPipelineDescriptor, NSApplication, NSApplicationActivationPolicy, NSBackingStoreType,
    NSString, NSURL, NSWindow, NSWindowStyleMask, cls, init_objc,
};

pub use crimes::{
    AllocObj, InstancePtr, Sel, StaticClsPtr, TypedIvar, TypedPtr, make_subclass, register_class,
};
