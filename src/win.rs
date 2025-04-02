mod objc;

use objc::{
    CGPoint, CGRect, CGSize, NSApplication, NSApplicationActivationPolicy, NSBackingStoreType,
    NSString, NSWindow, NSWindowStyleMask,
};

pub fn init() {
    NSApplication::init();
    NSWindow::init();
    NSString::init();

    let app = NSApplication::shared_app();
    app.set_activation_policy(NSApplicationActivationPolicy::Regular);

    let rect = CGRect {
        origin: CGPoint { x: 200.0, y: 200.0 },
        size: CGSize {
            width: 800.0,
            height: 600.0,
        },
    };
    let style_mask = NSWindowStyleMask::TITLED
        | NSWindowStyleMask::CLOSABLE
        | NSWindowStyleMask::MINIATURIZABLE
        | NSWindowStyleMask::RESIZABLE;
    let title = NSString::new(c"Hello, World!");

    let win = NSWindow::alloc_init(rect, style_mask, NSBackingStoreType::Buffered, false);
    win.set_title(title);
    win.set_visibility(true);
    win.set_main();
    app.run();
}
