use crate::{
    draw::DrawState,
    error::OrDie,
    objc::{
        self, NSString, OPtr, Sel, TypedCls, TypedObj,
        wrappers::{
            CGPoint, CGRect, CGSize, MTKView, MTLDevice, NSApplication,
            NSApplicationActivationPolicy, NSBackingStoreType, NSEvent, NSMenu, NSMenuItem,
            NSResponder, NSWindow, NSWindowDelegate, NSWindowStyleMask, sel,
        },
    },
};

extern "C" fn win_should_close(_slf: TypedObj<WinState>, _sel: Sel, sender: OPtr) -> bool {
    NSApplication::IPtr::shared().stop(sender);
    true
}

extern "C" fn win_did_resize(mut slf: TypedObj<WinState>, _sel: Sel, _notify: OPtr) {
    let state = slf.get_inner();
    let rect = state.win.content_rect();
    let aspect = state.win.content_aspect_ratio();
    let x = (rect.size.width / aspect.width).round();
    state.win.set_content_size(CGSize {
        width: aspect.width * x,
        height: aspect.height * x,
    })
}

extern "C" fn key_down(_slf: TypedObj<()>, _sel: Sel, key: NSEvent::IPtr) {
    dbg!(key.chars());
    dbg!(key.chars_ignore_mod());
    dbg!(key.key_code());
}

extern "C" fn flags_changed(_slf: TypedObj<()>, _sel: Sel, flags: NSEvent::IPtr) {
    dbg!(flags.key_code());
    dbg!(flags.mod_flags());
}

#[derive(Debug)]
struct WinState {
    win: NSWindow::IPtr,
}

impl WinState {
    fn init_delegate_cls() -> TypedCls<WinState, NSWindowDelegate::PPtr> {
        let cls = TypedCls::make_class(c"NSWindowDelegateWithWinState").or_die("UNREACHABLE");
        NSWindowDelegate::PPtr::implement(&cls, win_should_close, win_did_resize);
        cls
    }
}

#[derive(Debug)]
struct MyNSWindow;

impl MyNSWindow {
    fn init_as_subclass() -> TypedCls<MyNSWindow, NSWindow::IPtr> {
        let cls = TypedCls::make_subclass(NSWindow::cls(), c"MyNSWindow").or_die("UNREACHABLE");
        NSResponder::override_accepts_first_responder_as_true(cls.cls());
        NSResponder::override_key_down(cls.cls(), key_down);
        NSResponder::override_flag_changed(cls.cls(), flags_changed);
        cls
    }
}

fn setup_main_menu(app: NSApplication::IPtr) {
    let main_menu = NSMenu::IPtr::new(c"MainMenu");
    let app_menu_item = NSMenuItem::IPtr::new(c"AppMenu", None, c"");
    let app_menu = NSMenu::IPtr::new(c"AppMenu");
    let quit_item = NSMenuItem::IPtr::new(c"Quit", Some(sel::stop_.sel()), c"q");
    app_menu.add_item(quit_item);
    app_menu_item.set_submenu(app_menu);
    main_menu.add_item(app_menu_item);
    app.set_main_menu(main_menu);
}

pub fn init() {
    objc::init_objc();

    let win_dele_cls = WinState::init_delegate_cls();
    let view_dele_cls = DrawState::init_delegate_cls();
    let my_win = MyNSWindow::init_as_subclass();

    let app = NSApplication::IPtr::shared();
    app.set_activation_policy(NSApplicationActivationPolicy::Regular);
    setup_main_menu(app);

    let size = CGSize {
        width: 160.0,
        height: 100.0,
    };

    let rect = CGRect {
        origin: CGPoint { x: 200.0, y: 200.0 },
        size,
    };
    let style_mask = NSWindowStyleMask::TITLED
        | NSWindowStyleMask::CLOSABLE
        | NSWindowStyleMask::MINIATURIZABLE
        | NSWindowStyleMask::RESIZABLE;
    let title = NSString::IPtr::new(c"bang!");

    let win = my_win.alloc_upcasted(MyNSWindow);
    let win = NSWindow::IPtr::init(win, rect, style_mask, NSBackingStoreType::Buffered, false);

    let device = MTLDevice::PPtr::get_default();

    let alloc = MTKView::alloc();
    let view = MTKView::IPtr::init(alloc, rect, device);
    let dele = DrawState::new(device, view.color_pixel_fmt());
    view.set_preferred_fps(120);
    view.set_delegate(view_dele_cls.alloc_init_upcasted(dele));
    win.set_delegate(win_dele_cls.alloc_init_upcasted(WinState { win }));
    win.set_content_view(view);
    win.set_title(title);
    win.set_is_visible(true);
    win.set_main();
    win.center();
    win.set_content_min_size(size);
    win.set_content_aspect_ratio(size);
    app.run();
}
