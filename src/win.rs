use crate::objc::{
    self, CGPoint, CGRect, CGSize, MTKView, MTLCreateSystemDefaultDevice, NSApplication,
    NSApplicationActivationPolicy, NSBackingStoreType, NSString, NSWindow, NSWindowStyleMask,
};

pub fn init() {
    objc::init_base();
    NSApplication::init();
    NSWindow::init();
    MTKView::init();

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

    let frame = CGRect {
        origin: CGPoint { x: 300.0, y: 300.0 },
        size: CGSize {
            width: 100.0,
            height: 100.0,
        },
    };
    let device = MTLCreateSystemDefaultDevice();
    device.check_null();
    println!("{:?}", device.max_tg_mem_len());
    //let view = MTKView::new(frame, device);
    println!("success");

    win.set_title(title);
    win.set_visibility(true);
    win.set_main();
    win.center();
    app.run();
}
