use std::{ffi::CString, marker::PhantomData};

use bang_core::{
    Config,
    ffi::RtCtx,
    input::{Key, KeyState},
};
use bang_rt_common::{die, draw::DrawReceiver, end::Ender, error::OrDie, input::InputGatherer};

use crate::{
    RtState,
    draw::DrawState,
    objc::{
        NSString, OPtr, Sel, TypedCls, TypedObj, TypedPtr,
        wrappers::{
            CGPoint, CGRect, CGSize, MTKView, NSApplication, NSApplicationActivationPolicy,
            NSApplicationDelegate, NSApplicationTerminateReply, NSBackingStoreType, NSEvent,
            NSMenu, NSMenuItem, NSResponder, NSWindow, NSWindowDelegate, NSWindowStyleMask, sel,
        },
    },
    timer::TimeConverter,
};

extern "C" fn app_should_terminate(
    _slf: TypedObj<AppState>,
    _sel: Sel,
    sender: OPtr,
) -> NSApplicationTerminateReply {
    NSApplication::IPtr::shared().stop(sender);
    NSApplicationTerminateReply::Cancel
}

extern "C" fn win_should_close(_slf: TypedObj<WinState>, _sel: Sel, sender: OPtr) -> bool {
    NSApplication::IPtr::shared().terminate(sender);
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

extern "C" fn key_down(mut slf: TypedObj<MyNSWindow>, _sel: Sel, ev: NSEvent::IPtr) {
    let my_win = slf.get_inner();
    let key = Key::from_code(ev.key_code());
    let timestamp = my_win.time_converter.sys_to_instant(ev.timestamp());
    my_win.gatherer.update(key, KeyState::Pressed, timestamp);
}

extern "C" fn key_up(mut slf: TypedObj<MyNSWindow>, _sel: Sel, ev: NSEvent::IPtr) {
    let my_win = slf.get_inner();
    let key = Key::from_code(ev.key_code());
    let timestamp = my_win.time_converter.sys_to_instant(ev.timestamp());
    my_win.gatherer.update(key, KeyState::Released, timestamp);
}

extern "C" fn flags_changed(mut slf: TypedObj<MyNSWindow>, _sel: Sel, flags: NSEvent::IPtr) {
    let _my_win = slf.get_inner(); // TODO
    dbg!(flags.key_code());
    dbg!(flags.mod_flags());
}

#[derive(Debug)]
struct AppState {}

impl AppState {
    fn init_delegate_cls() -> TypedCls<AppState, NSApplicationDelegate::PPtr> {
        let cls =
            TypedCls::make_class(c"NSWindowDelegateWithWinState").or_(die!("Already exists?"));
        NSApplicationDelegate::PPtr::implement(&cls, app_should_terminate);
        cls
    }
}

#[derive(Debug)]
struct WinState {
    win: NSWindow::IPtr,
}

impl WinState {
    fn init_delegate_cls() -> TypedCls<WinState, NSWindowDelegate::PPtr> {
        let cls =
            TypedCls::make_class(c"NSApplicationDelegateWithAppState").or_(die!("Already exists?"));
        NSWindowDelegate::PPtr::implement(&cls, win_should_close, win_did_resize);
        cls
    }
}

#[derive(Debug)]
struct MyNSWindow<'l> {
    gatherer: InputGatherer<'l>,
    time_converter: TimeConverter,
}

impl<'l> MyNSWindow<'l> {
    fn new(gatherer: InputGatherer<'l>) -> Self {
        let time_converter = TimeConverter::new();
        Self {
            gatherer,
            time_converter,
        }
    }

    fn init_as_subclass() -> TypedCls<MyNSWindow<'l>, NSWindow::IPtr> {
        let cls = TypedCls::make_subclass(NSWindow::cls(), c"MyNSWindow").or_(die!("UNREACHABLE"));
        NSResponder::override_accepts_first_responder_as_true(cls.cls());
        NSResponder::override_key_down(cls.cls(), key_down);
        NSResponder::override_key_up(cls.cls(), key_up);
        NSResponder::override_flag_changed(cls.cls(), flags_changed);
        cls
    }
}

fn setup_main_menu(app: NSApplication::IPtr) {
    let main_menu = NSMenu::IPtr::new(c"MainMenu");
    let app_menu_item = NSMenuItem::IPtr::new(c"AppMenu", None, c"");
    let app_menu = NSMenu::IPtr::new(c"AppMenu");
    let quit_item = NSMenuItem::IPtr::new(c"Quit", Some(sel::terminate_.sel()), c"q");
    app_menu.add_item(quit_item);
    app_menu_item.set_submenu(app_menu);
    main_menu.add_item(app_menu_item);
    app.set_main_menu(Some(main_menu));
}

pub struct Window<'l> {
    app: NSApplication::IPtr,
    _win: NSWindow::IPtr,
    _marker: PhantomData<&'l ()>,
}

impl<'l> Window<'l> {
    pub fn init(
        rt_ctx: &mut RtCtx,
        input_gatherer: InputGatherer,
        draw_receiver: DrawReceiver<'l>,
        config: &'l Config,
        ender: &'l Ender,
    ) -> Self {
        let rt = RtState::unwrap_from(rt_ctx);

        let app_dele_cls = AppState::init_delegate_cls();
        let win_dele_cls = WinState::init_delegate_cls();
        let view_dele_cls = DrawState::init_delegate_cls();
        let my_win = MyNSWindow::init_as_subclass();

        let app = NSApplication::IPtr::shared();
        app.set_delegate(app_dele_cls.alloc_init_upcasted(AppState {}));
        app.set_activation_policy(NSApplicationActivationPolicy::Regular);
        setup_main_menu(app);

        let size = CGSize {
            width: config.resolution.0 as f64 / 2.0,
            height: config.resolution.1 as f64 / 2.0,
        };

        let rect = CGRect {
            origin: CGPoint { x: 1.0, y: 1.0 },
            size,
        };
        let style_mask = NSWindowStyleMask::TITLED
            | NSWindowStyleMask::CLOSABLE
            | NSWindowStyleMask::MINIATURIZABLE
            | NSWindowStyleMask::RESIZABLE;
        let title = CString::new(config.name).or_(die!("Failed to create CString"));
        let title = NSString::IPtr::new(&title);

        let win = my_win.alloc_upcasted(MyNSWindow::new(input_gatherer));
        let win = NSWindow::IPtr::init(win, rect, style_mask, NSBackingStoreType::Buffered, false);

        let alloc = MTKView::alloc();
        let view = MTKView::IPtr::init(alloc, rect, rt.device);
        let dele = DrawState::new(
            rt.device,
            view.color_pixel_fmt(),
            draw_receiver,
            config,
            ender,
        );
        view.set_preferred_fps(120);
        view.set_delegate(view_dele_cls.alloc_init_upcasted(dele));
        win.set_delegate(win_dele_cls.alloc_init_upcasted(WinState { win }));
        win.set_content_view(view);
        win.set_title(title);
        win.set_is_visible(true);
        win.set_main();
        win.set_content_min_size(size);
        win.set_content_aspect_ratio(size);
        win.set_content_size(CGSize {
            width: size.width * config.scale as f64,
            height: size.height * config.scale as f64,
        });
        win.center();
        Window {
            app,
            _win: win,
            _marker: PhantomData,
        }
    }

    pub fn run(&self) {
        self.app.run();
    }

    pub fn notify_end(ender: &Ender) {
        if ender.should_end() {
            let app = NSApplication::IPtr::shared();
            if app.running() {
                app.terminate(app.obj());
            }
        }
    }
}
