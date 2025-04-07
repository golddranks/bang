use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    error::OrDie,
    objc::{
        self, CGPoint, CGRect, CGSize, InstancePtr, MTKView, MTLBuffer, MTLClearColor,
        MTLCommandQueue, MTLDevice, MTLPrimitiveType, MTLRenderPipelineDescriptor,
        MTLRenderPipelineState, MTLResourceOptions, NSApplication, NSApplicationActivationPolicy,
        NSBackingStoreType, NSString, NSUrl, NSWindow, NSWindowStyleMask, Sel, StaticClsPtr,
        TypedIvar, TypedPtr, cls,
    },
};

extern "C" fn window_should_close_override(sender: NSWindow, _sel: Sel) -> bool {
    println!("Window closed!");
    NSApplication::shared().stop(sender.obj());
    true
}

#[derive(Debug)]
struct CustomMTKView {
    view: MTKView,
    cmd_queue: TypedIvar<MTLCommandQueue>,
    vtex_buf: TypedIvar<MTLBuffer>,
    rend_pl_state: TypedIvar<MTLRenderPipelineState>,
}

static FRAME: AtomicU64 = AtomicU64::new(0);

static CUSTOM_VIEW_CLS: StaticClsPtr = StaticClsPtr::new(c"CustomMTKView");

impl CustomMTKView {
    fn init() {
        let custom_view_cls =
            objc::make_subclass(cls::MTKView.cls(), c"CustomMTKView").or_die("UNREACHABLE");
        unsafe {
            custom_view_cls.add_ivar::<MTLCommandQueue>(c"cmd_queue", c"@");
            custom_view_cls.add_ivar::<MTLBuffer>(c"vtex_buf", c"@");
            custom_view_cls.add_ivar::<MTLRenderPipelineState>(c"rend_pl_state", c"@");
        };
        let custom_view_cls = objc::register_class(custom_view_cls);
        CUSTOM_VIEW_CLS.init_with(custom_view_cls);
    }

    unsafe fn new(view: MTKView) -> Self {
        let cmd_queue = CUSTOM_VIEW_CLS
            .cls()
            .ivar(c"cmd_queue")
            .or_die("UNREACHABLE");
        let vtex_buf = CUSTOM_VIEW_CLS
            .cls()
            .ivar(c"vtex_buf")
            .or_die("UNREACHABLE");
        let rend_pl_state = CUSTOM_VIEW_CLS
            .cls()
            .ivar(c"rend_pl_state")
            .or_die("UNREACHABLE");
        let cmd_queue = unsafe { TypedIvar::new(cmd_queue) };
        let vtex_buf = unsafe { TypedIvar::new(vtex_buf) };
        let rend_pl_state = unsafe { TypedIvar::new(rend_pl_state) };
        Self {
            view,
            cmd_queue,
            vtex_buf,
            rend_pl_state,
        }
    }

    fn cmd_queue(&self) -> MTLCommandQueue {
        self.cmd_queue.get(self.view.obj())
    }

    fn vtex_buf(&self) -> MTLBuffer {
        self.vtex_buf.get(self.view.obj())
    }

    fn rend_pl_state(&self) -> MTLRenderPipelineState {
        self.rend_pl_state.get(self.view.obj())
    }
}

extern "C" fn draw_rect_override(view: MTKView, _sel: Sel, _dirty_rect: CGRect) {
    let cview = unsafe { CustomMTKView::new(view) };
    let frame = FRAME.fetch_add(1, Ordering::Relaxed);
    let phase = frame % 100;
    if phase == 0 {
        println!("Frame: {}", frame);
    }
    let phase_float = 0.04 * phase as f32 - 2.0;

    let pass_desc = view.current_rendpass_desc().or_die("rendpass_desc");
    pass_desc
        .color_attach()
        .at(0)
        .set_clear_color(MTLClearColor::new(0.0, phase_float as f64, 0.0, 1.0));
    let cmd_buf = cview.cmd_queue().cmd_buf().or_die("cmd_buf");

    let rencoder = cmd_buf.rencoder_with_desc(pass_desc).or_die("rencoder");
    rencoder.set_rend_pl_state(cview.rend_pl_state());
    rencoder.set_vtex_buf(cview.vtex_buf(), 0, 0);
    rencoder.set_vtex_bytes(&phase_float.to_le_bytes(), 1);
    rencoder.draw_primitive(MTLPrimitiveType::Triangle, 0, 3);
    rencoder.end();

    let drawable = view.current_drawable().or_die("drawable");
    cmd_buf.present_drawable(drawable);
    cmd_buf.commit();
}

pub fn init() {
    objc::init_objc();

    NSWindow::override_window_should_close(window_should_close_override);
    MTKView::override_draw_rect(draw_rect_override);
    CustomMTKView::init();

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

    let view = init_mtl_pipeline();
    win.set_content_view(view);
    win.set_title(title);
    win.set_is_visible(true);
    win.set_main();
    win.center();
    app.run();
}

// The alignment is required because on GPU, we are using 4xf32 SIMD vectors
#[repr(C, align(16))]
struct Vertex {
    color: [f32; 4],
    pos: [f32; 2],
}

impl Vertex {
    fn new(color: [f32; 4], pos: [f32; 2]) -> Self {
        Vertex { color, pos }
    }
}

fn init_mtl_pipeline() -> MTKView {
    let frame = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: CGSize {
            width: 800.0,
            height: 600.0,
        },
    };
    let device = MTLDevice::get_default();
    let cmd_queue = device
        .new_cmd_queue()
        .or_die("new_cmd_queue: Failed to create command queue");
    let alloc = unsafe { CUSTOM_VIEW_CLS.cls().alloc::<MTKView>() };
    let view = MTKView::init(alloc, frame, device);
    view.set_preferred_fps(120);
    let cview = unsafe { CustomMTKView::new(view) };
    cview.cmd_queue.set(view.obj(), cmd_queue);

    dbg!(size_of::<Vertex>());

    let vtex: [Vertex; 3] = [
        Vertex::new([1.0, 0.0, 0.0, 1.0], [-1.0, -1.0]),
        Vertex::new([0.1, 1.0, 0.0, 1.0], [0.0, 1.0]),
        Vertex::new([0.0, 0.0, 1.0, 1.0], [1.0, -1.0]),
    ];
    let vtex_buf = device
        .new_buf(&vtex, MTLResourceOptions::DEFAULT)
        .or_die("new_buf: Failed to create vertex buffer");
    cview.vtex_buf.set(view.obj(), vtex_buf);

    let desc = MTLRenderPipelineDescriptor::new();

    let lib = device
        .new_lib_with_url(NSUrl::new(c"target/shaders.metallib"))
        .or_die("new_lib_with_url: Failed to create library");

    desc.set_vtex_fn(lib.new_fn(c"vertexShader"));
    desc.set_frag_fn(lib.new_fn(c"fragmentShader"));
    let attach = desc.color_attach().at(0);
    attach.set_pixel_fmt(view.color_pixel_fmt());

    let rend_pl_state = device
        .new_rend_pl_state(desc)
        .or_die("new_rend_pl_state: Failed to create render pipeline state");
    cview.rend_pl_state.set(view.obj(), rend_pl_state);

    view
}
