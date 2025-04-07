use crate::{
    error::OrDie,
    objc::{
        self, CGPoint, CGRect, CGSize, InstancePtr, MTKView, MTLBuffer, MTLCommandQueue, MTLDevice,
        MTLRenderPipelineDescriptor, NSApplication, NSApplicationActivationPolicy,
        NSBackingStoreType, NSString, NSURL, NSWindow, NSWindowStyleMask, Sel, StaticClsPtr,
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
}

impl CustomMTKView {
    unsafe fn new(view: MTKView) -> Self {
        let cmd_queue = CUSTOM_VIEW_CLS
            .cls()
            .ivar(c"cmd_queue")
            .or_die("UNREACHABLE");
        let vtex_buf = CUSTOM_VIEW_CLS
            .cls()
            .ivar(c"vtex_buf")
            .or_die("UNREACHABLE");
        let cmd_queue = unsafe { TypedIvar::new(cmd_queue) };
        let vtex_buf = unsafe { TypedIvar::new(vtex_buf) };
        Self {
            view,
            cmd_queue,
            vtex_buf,
        }
    }

    fn cmd_queue(&self) -> MTLCommandQueue {
        self.cmd_queue.get(self.view.obj())
    }
}

extern "C" fn draw_rect_override(view: MTKView, _sel: Sel, _dirty_rect: CGRect) {
    let cview = unsafe { CustomMTKView::new(view) };
    let device = view.device().or_die("device");
    let cmd_queue = cview.cmd_queue();

    let pass_desc = view.current_rendpass_desc().or_die("rendpass_desc");
    let drawable = view.current_drawable().or_die("drawable");
    let cmd_buf = cmd_queue.cmd_buf().or_die("cmd_buf");

    /*
    let vtex_fn = msg3::<Obj, NSString, Ptr, Ptr>(
        &device,
        &MTL_DEVICE_SEL_NEW_LIB,
        NSString::new(VTEX_SHADER),
        null_mut(),
        null_mut(),
    )
    .or_die("vtex_fn");
    let frag_fn = msg3::<Obj, NSString, Ptr, Ptr>(
        &device,
        &MTL_DEVICE_SEL_NEW_LIB,
        NSString::new(FRAG_SHADER),
        null_mut(),
        null_mut(),
    )
    .or_die("frag_fn"); */
    let rencoder = cmd_buf.rencoder_with_desc(pass_desc).or_die("rencoder");
    //let pipe_desc = msg0::<Obj>(&device, &MTL_DEVICE_SEL_NEW_PIPE_DESC).or_die("pipe_desc");
    //msg1::<(), _>(&pipe_desc, &MTL_PIPE_DESC_SEL_ADD_VTEX_FN, vtex_fn);
    //msg1::<(), _>(&pipe_desc, &MTL_PIPE_DESC_SEL_ADD_FRAG_FN, frag_fn);
    //let pipe_state = msg1::<Obj, _>(&device, &MTL_DEVICE_SEL_NEW_PIPE_STATE, pipe_desc);

    rencoder.end();

    //msg1::<(), _>(&rencoder, &MTL_RENCODER_SEL_SET_PIPE_STATE, pipe_state);
    //msg1::<(), _>(&rencoder, &MTL_RENCODER_SEL_SET_VTEX_BUF, vtex_buf);
    /*msg3::<(), MTLPrimitiveType, NSUInteger, NSUInteger>(
        &rencoder,
        &MTL_RENCODER_SEL_DRAW_PRIMITIVES,
        MTLPrimitiveType::Triangle,
        0,
        3,
    );*/
    cmd_buf.present_drawable(drawable);
    cmd_buf.commit();

    println!("draw_rect_override completed!");
}

/*
const VTEX_SHADER: &CStr = cr##"
#include <metal_stdlib>
using namespace metal;

struct VertexIn {
    float4 position [[attribute(0)]];
};

struct VertexOut {
    float4 position [[position]];
};

vertex VertexOut vertex_main(VertexIn in [[stage_in]]) {
    VertexOut out;
    out.position = in.position; // Simply pass through the position
    return out;
}
\0"##;

const FRAG_SHADER: &CStr = cr##"
#include <metal_stdlib>
using namespace metal;

fragment float4 fragment_main() {
    return float4(1.0, 0.0, 0.0, 1.0); // Red color for the triangle
}
\0"##; */

static CUSTOM_VIEW_CLS: StaticClsPtr = StaticClsPtr::new(c"CustomMTKView");

fn init_custom_view_cls() {
    let custom_view_cls =
        objc::make_subclass(cls::MTKView.cls(), c"CustomMTKView").or_die("UNREACHABLE");
    unsafe {
        custom_view_cls.add_ivar::<MTLCommandQueue>(c"cmd_queue", c"@");
        custom_view_cls.add_ivar::<MTLBuffer>(c"vtex_buf", c"@")
    };
    let custom_view_cls = objc::register_class(custom_view_cls);
    CUSTOM_VIEW_CLS.init_with(custom_view_cls);
}

pub fn init() {
    objc::init_objc();

    NSWindow::override_window_should_close(window_should_close_override);
    MTKView::override_draw_rect(draw_rect_override);
    init_custom_view_cls();

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

fn init_mtl_pipeline() -> MTKView {
    let frame = CGRect {
        origin: CGPoint { x: 300.0, y: 300.0 },
        size: CGSize {
            width: 100.0,
            height: 100.0,
        },
    };
    let device = MTLDevice::get_default();
    let cmd_queue = device
        .new_cmd_queue()
        .or_die("new_cmd_queue: Failed to create command queue");
    let alloc = unsafe { CUSTOM_VIEW_CLS.cls().alloc::<MTKView>() };
    let view = MTKView::init(alloc, frame, device);
    let cview = unsafe { CustomMTKView::new(view) };
    cview.cmd_queue.set(view.obj(), cmd_queue);
    let vtex: [f32; 9] = [
        0.0, 0.5, 0.0, // Top vertex
        -0.5, -0.5, 0.0, // Bottom-left vertex
        0.5, -0.5, 0.0, // Bottom-right vertex
    ];
    let vtex_buf = device
        .new_buf(&vtex)
        .or_die("new_buf: Failed to create vertex buffer");
    cview.vtex_buf.set(view.obj(), vtex_buf);

    let desc = MTLRenderPipelineDescriptor::new();

    let lib = device
        .new_lib_with_url(NSURL::new(c"target/shaders.metallib"))
        .or_die("new_lib_with_url: Failed to create library");

    desc.set_vtex_fn(lib.new_fn(c"vertexShader"));
    desc.set_frag_fn(lib.new_fn(c"fragmentShader"));

    let rend_pl_state = device
        .new_rend_pl_state(desc)
        .or_die("new_rend_pl_state: Failed to create render pipeline state");

    view
}
