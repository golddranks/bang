// We only support 64-bit macOS on Apple Silicon
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
mod crimes;
pub mod wrappers;

pub use crimes::{NSString, NSUInteger, OPtr, Sel, TypedCls, TypedObj, TypedPtr};
pub use wrappers::init_objc;
