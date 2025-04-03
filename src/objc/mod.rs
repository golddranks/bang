// We only support 64-bit macOS on Apple Silicon
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
mod crimes;
mod wrappers;

pub use wrappers::{
    CGPoint, CGRect, CGSize, MTKView, MTLDevice, NSApplication, NSApplicationActivationPolicy,
    NSBackingStoreType, NSString, NSWindow, NSWindowStyleMask, init_base,
};
