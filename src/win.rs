mod objc;

use objc::{
    CGPoint, CGRect, CGSize, NSApplicationActivationPolicy, NSApplicationCls, NSBackingStoreType,
    NSStringCls, NSWindowCls, NSWindowStyleMask,
};

pub fn init() {
    let app_cls = NSApplicationCls::init();
    let win_cls = NSWindowCls::init();
    let str_cls = NSStringCls::init();

    let app = app_cls.shared_app();
    app.set_activation_policy(NSApplicationActivationPolicy::Regular);

    let rect = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: CGSize {
            width: 800.0,
            height: 600.0,
        },
    };

    let style_mask = NSWindowStyleMask::TITLED
        | NSWindowStyleMask::CLOSABLE
        | NSWindowStyleMask::MINIATURIZABLE
        | NSWindowStyleMask::RESIZABLE;
    let title = str_cls.new(c"Hello, World!");

    let win = win_cls.alloc_init(&app_cls, rect, style_mask, NSBackingStoreType::Buffered, false);
    win.set_title(title);
    win.set_visibility(true);
    win.set_main();
    app.run();
}
