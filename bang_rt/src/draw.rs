use std::{
    ffi::CString,
    ops::Not,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

use bang_core::draw::{DRAW_FRAME_DUMMY, DrawFrame};

use crate::{
    alloc::AllocRetirer,
    error::OrDie,
    objc::{
        NSString, Sel, TypedCls, TypedObj,
        wrappers::{
            CGSize, MTKView, MTKViewDelegate, MTLBuffer, MTLClearColor, MTLCommandQueue,
            MTLCompileOptions, MTLDevice, MTLPixelFormat, MTLPrimitiveType,
            MTLRenderPipelineDescriptor, MTLRenderPipelineState, MTLResourceOptions, NSUrl,
        },
    },
};

#[derive(Debug)]
pub struct SharedDrawState<'l> {
    fresh: AtomicPtr<DrawFrame<'l>>,
}

impl<'l> SharedDrawState<'l> {
    pub fn new() -> Self {
        Self {
            fresh: AtomicPtr::new(null_mut()),
        }
    }
}

#[derive(Debug)]
pub struct DrawSender<'l> {
    shared: &'l SharedDrawState<'l>,
}

#[derive(Debug)]
pub struct DrawReceiver<'l> {
    shared: &'l SharedDrawState<'l>,
    retirer: AllocRetirer<'l>,
    fresh: &'l DrawFrame<'l>,
}

pub fn new_draw_pair<'l>(
    shared: &'l mut SharedDrawState<'l>,
    retirer: AllocRetirer<'l>,
) -> (DrawSender<'l>, DrawReceiver<'l>) {
    let shared = &*shared;
    let sender = DrawSender { shared };
    let receiver = DrawReceiver {
        shared,
        retirer,
        fresh: &DRAW_FRAME_DUMMY,
    };
    (sender, receiver)
}

impl<'l> DrawSender<'l> {
    pub fn send<'f>(&mut self, frame: &'f mut DrawFrame<'f>) {
        let perennial_frame = &raw mut *frame as *mut DrawFrame<'l>;
        self.shared.fresh.swap(perennial_frame, Ordering::Release);
    }
}

impl<'l> DrawReceiver<'l> {
    fn get_fresh<'s>(&'s mut self) -> &'s DrawFrame<'s> {
        let freshest = self.shared.fresh.swap(null_mut(), Ordering::Acquire);
        if freshest.is_null().not() {
            let retired_seq = self.fresh.alloc_seq();
            self.fresh = unsafe { &mut *freshest };
            self.retirer.retire(retired_seq);
        }
        self.fresh
    }
}

#[derive(Debug)]
pub struct DrawState<'l> {
    draw_receiver: DrawReceiver<'l>,
    cmd_queue: MTLCommandQueue::PPtr,
    vtex_buf: MTLBuffer::PPtr,
    rend_pl_state: MTLRenderPipelineState::PPtr,
    frame: usize,
}

extern "C" fn draw(mut dele: TypedObj<DrawState>, _sel: Sel, view: MTKView::IPtr) {
    let state = dele.get_inner();
    let frame = state.draw_receiver.get_fresh();
    dbg!(frame);

    let phase = state.frame % 100;
    let color_phase = 0.01 * phase as f64;
    let pos_phase = 0.04 * phase as f32 - 2.0;

    let pass_desc = view.current_rendpass_desc().or_die("rendpass_desc");
    pass_desc
        .color_attach()
        .at(0)
        .set_clear_color(MTLClearColor::new(color_phase, 0.0, 0.0, 1.0));
    let cmd_buf = state.cmd_queue.cmd_buf().or_die("cmd_buf");

    let rencoder = cmd_buf.rencoder_with_desc(pass_desc).or_die("rencoder");
    rencoder.set_rend_pl_state(state.rend_pl_state);
    rencoder.set_vtex_buf(state.vtex_buf, 0, 0);
    rencoder.set_vtex_bytes(&pos_phase.to_le_bytes(), 1);
    rencoder.draw_primitive(MTLPrimitiveType::Triangle, 0, 3);
    rencoder.end();

    let drawable = view.current_drawable().or_die("drawable");
    cmd_buf.present_drawable(drawable);
    cmd_buf.commit();

    state.frame += 1;
}

extern "C" fn size_change(
    _slf: TypedObj<DrawState>,
    _sel: Sel,
    _view: MTKView::IPtr,
    size: CGSize,
) {
    eprintln!("size change called?! {:?}", size);
}

impl<'l> DrawState<'l> {
    pub fn init_delegate_cls() -> TypedCls<DrawState<'l>, MTKViewDelegate::PPtr> {
        let cls = TypedCls::make_class(c"MTKViewDelegateWithDrawState").or_die("UNREACHABLE");
        MTKViewDelegate::PPtr::implement(&cls, draw, size_change);
        cls
    }

    pub fn new(
        device: MTLDevice::PPtr,
        pixel_fmt: MTLPixelFormat,
        draw_receiver: DrawReceiver<'l>,
    ) -> Self {
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

        let desc = MTLRenderPipelineDescriptor::IPtr::new();

        let options = MTLCompileOptions::IPtr::new();

        let lib = match device.new_lib_with_url(NSUrl::IPtr::new(c"target/shaders.metallib")) {
            Ok(lib) => lib,
            Err(_) => {
                eprintln!("Couldn't find precompiled shaders, compiling from source...");
                let mut source = std::fs::read_to_string("bang_rt/src/shaders.metal")
                    .or_die("Failed to read shader source");
                source.push('\0');
                let source = NSString::IPtr::new(
                    &CString::from_vec_with_nul(source.into_bytes()).expect("UNREACHABLE"),
                );
                let lib = device
                    .new_lib_from_source(source, options)
                    .or_die("failed to create library");
                eprintln!("Compiled!");
                lib
            }
        };

        desc.set_vtex_fn(lib.new_fn(c"vertexShader"));
        desc.set_frag_fn(lib.new_fn(c"fragmentShader"));
        let attach = desc.color_attach().at(0);

        attach.set_pixel_fmt(pixel_fmt);

        let rend_pl_state = device
            .new_rend_pl_state(desc)
            .or_die("new_rend_pl_state: Failed to create render pipeline state");

        Self {
            draw_receiver,
            cmd_queue,
            vtex_buf,
            rend_pl_state,
            frame: 0,
        }
    }
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
