use crate::{
    draw::DrawState,
    objc::{
        self, CGPoint, CGRect, CGSize, InstancePtr, MTKView, MTLDevice, NSApplication,
        NSApplicationActivationPolicy, NSBackingStoreType, NSString, NSWindow, NSWindowStyleMask,
        Obj, Sel,
    },
};

extern "C" fn window_should_close_override(_slf: NSWindow, _sel: Sel, sender: Obj) -> bool {
    println!("Window closed!");
    NSApplication::shared().stop(sender);
    true
}

pub fn init() {
    objc::init_objc();

    NSWindow::override_window_should_close(window_should_close_override);
    let dele_cls = DrawState::init_delegate_cls();

    let app = NSApplication::shared();
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

    let win = NSWindow::alloc();
    let win = NSWindow::init(win, rect, style_mask, NSBackingStoreType::Buffered, false);

    let device = MTLDevice::get_default();

    let alloc = MTKView::alloc();
    let view = MTKView::init(alloc, rect, device);
    let dele = DrawState::new(device, view.color_pixel_fmt());
    view.set_preferred_fps(120);
    view.set_delegate(dele_cls.new_untyped(dele));

    win.set_content_view(view);
    win.set_title(title);
    win.set_is_visible(true);
    win.set_main();
    win.center();
    app.run();
}
