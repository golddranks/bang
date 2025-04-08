use crate::{
    draw::DrawState,
    error::OrDie,
    objc::{
        self, OPtr, Sel, TypedCls, TypedObj,
        wrappers::{
            CGPoint, CGRect, CGSize, MTKView, MTLDevice, NSApplication,
            NSApplicationActivationPolicy, NSBackingStoreType, NSString, NSWindow,
            NSWindowDelegate, NSWindowStyleMask,
        },
    },
};

extern "C" fn window_should_close(_slf: TypedObj<WinState>, _sel: Sel, sender: OPtr) -> bool {
    println!("Window closed!");
    NSApplication::IPtr::shared().stop(sender);
    true
}

#[derive(Debug)]
struct WinState;

impl WinState {
    fn init_delegate_cls() -> TypedCls<WinState, NSWindowDelegate::PPtr> {
        let cls = TypedCls::init(c"NSWindowDelegateWithWinState").or_die("UNREACHABLE");
        NSWindowDelegate::PPtr::implement(&cls, window_should_close);
        cls
    }
}

pub fn init() {
    objc::init_objc();

    let win_dele_cls = WinState::init_delegate_cls();
    let view_dele_cls = DrawState::init_delegate_cls();

    let app = NSApplication::IPtr::shared();
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
    let title = NSString::IPtr::new(c"Hello, World!");

    let win = NSWindow::alloc();
    let win = NSWindow::IPtr::init(win, rect, style_mask, NSBackingStoreType::Buffered, false);

    let device = MTLDevice::PPtr::get_default();

    let alloc = MTKView::alloc();
    let view = MTKView::IPtr::init(alloc, rect, device);
    let dele = DrawState::new(device, view.color_pixel_fmt());
    view.set_preferred_fps(120);
    view.set_delegate(view_dele_cls.new_untyped(dele));
    win.set_delegate(win_dele_cls.new_untyped(WinState));
    win.set_content_view(view);
    win.set_title(title);
    win.set_is_visible(true);
    win.set_main();
    win.center();
    app.run();
}
