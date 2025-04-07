use crate::{
    error::OrDie,
    objc::{
        self, CGPoint, CGRect, CGSize, InstancePtr, MTKView, MTLBuffer, MTLClearColor,
        MTLCommandQueue, MTLDevice, MTLPixelFormat, MTLPrimitiveType, MTLRenderPipelineDescriptor,
        MTLRenderPipelineState, MTLResourceOptions, NSApplication, NSApplicationActivationPolicy,
        NSBackingStoreType, NSString, NSUrl, NSWindow, NSWindowStyleMask, Obj, Sel,
        TypedMTKViewDelegate, TypedMTKViewDelegateCls,
    },
};

extern "C" fn window_should_close_override(_slf: NSWindow, _sel: Sel, sender: Obj) -> bool {
    println!("Window closed!");
    NSApplication::shared().stop(sender);
    true
}

#[derive(Debug)]
struct DrawState {
    cmd_queue: MTLCommandQueue,
    vtex_buf: MTLBuffer,
    rend_pl_state: MTLRenderPipelineState,
    frame: usize,
}

extern "C" fn draw(mut dele: TypedMTKViewDelegate<DrawState>, _sel: Sel, view: MTKView) {
    let this = dele.get_inner();

    let phase = this.frame % 100;
    if phase == 0 {
        println!("Frame: {}", this.frame);
    }
    let color_phase = 0.01 * phase as f64;
    let pos_phase = 0.04 * phase as f32 - 2.0;

    let pass_desc = view.current_rendpass_desc().or_die("rendpass_desc");
    pass_desc
        .color_attach()
        .at(0)
        .set_clear_color(MTLClearColor::new(color_phase, 0.0, 0.0, 1.0));
    let cmd_buf = this.cmd_queue.cmd_buf().or_die("cmd_buf");

    let rencoder = cmd_buf.rencoder_with_desc(pass_desc).or_die("rencoder");
    rencoder.set_rend_pl_state(this.rend_pl_state);
    rencoder.set_vtex_buf(this.vtex_buf, 0, 0);
    rencoder.set_vtex_bytes(&pos_phase.to_le_bytes(), 1);
    rencoder.draw_primitive(MTLPrimitiveType::Triangle, 0, 3);
    rencoder.end();

    let drawable = view.current_drawable().or_die("drawable");
    cmd_buf.present_drawable(drawable);
    cmd_buf.commit();
    this.frame += 1;
}

extern "C" fn size_change(
    _slf: TypedMTKViewDelegate<DrawState>,
    _sel: Sel,
    _view: MTKView,
    _size: CGSize,
) {
    dbg!("size change called?!");
}

impl DrawState {
    fn init_delegate_cls() -> TypedMTKViewDelegateCls<DrawState> {
        TypedMTKViewDelegateCls::init(c"MTKViewDelegateWithDrawState", draw, size_change)
    }

    fn new(device: MTLDevice, pixel_fmt: MTLPixelFormat) -> Self {
        let cmd_queue = device
            .new_cmd_queue()
            .or_die("new_cmd_queue: Failed to create command queue");

        let vtex: [Vertex; 3] = [
            Vertex::new([1.0, 0.0, 0.0, 1.0], [-1.0, -1.0]),
            Vertex::new([0.1, 1.0, 0.0, 1.0], [0.0, 1.0]),
            Vertex::new([0.0, 0.0, 1.0, 1.0], [1.0, -1.0]),
        ];
        let vtex_buf = device
            .new_buf(&vtex, MTLResourceOptions::DEFAULT)
            .or_die("new_buf: Failed to create vertex buffer");

        let desc = MTLRenderPipelineDescriptor::new();

        let lib = device
            .new_lib_with_url(NSUrl::new(c"target/shaders.metallib"))
            .or_die("new_lib_with_url: Failed to create library");

        desc.set_vtex_fn(lib.new_fn(c"vertexShader"));
        desc.set_frag_fn(lib.new_fn(c"fragmentShader"));
        let attach = desc.color_attach().at(0);

        attach.set_pixel_fmt(pixel_fmt);

        let rend_pl_state = device
            .new_rend_pl_state(desc)
            .or_die("new_rend_pl_state: Failed to create render pipeline state");

        Self {
            cmd_queue,
            vtex_buf,
            rend_pl_state,
            frame: 0,
        }
    }
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
