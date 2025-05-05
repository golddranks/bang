use std::ffi::CString;

use bang_core::{
    Config,
    draw::{AsBytes, Cmd},
};
use bang_rt_common::{die, draw::DrawReceiver, end::Ender, error::OrDie};

use crate::objc::{
    NSString, NSUInteger, Sel, TypedCls, TypedObj,
    wrappers::{
        CGSize, MTKView, MTKViewDelegate, MTLBuffer, MTLClearColor, MTLCommandQueue,
        MTLCompileOptions, MTLDevice, MTLOrigin, MTLPixelFormat, MTLPrimitiveType, MTLRegion,
        MTLRenderPipelineDescriptor, MTLRenderPipelineState, MTLResourceOptions, MTLSize,
        MTLTexture, MTLTextureDescriptor, MTLVertexDescriptor, MTLVertexFormat, NSUrl,
    },
};

#[derive(Debug)]
pub struct DrawState<'l> {
    draw_receiver: DrawReceiver<'l>,
    cmd_queue: MTLCommandQueue::PPtr,
    quad_vtex_buf: MTLBuffer::PPtr,
    quad_tex: MTLTexture::PPtr,
    quad_pal_buf: MTLBuffer::PPtr,
    dbg_buf: MTLBuffer::PPtr,
    rend_pl_state: MTLRenderPipelineState::PPtr,
    frame: usize,
    config: &'l Config,
    ender: &'l Ender,
}

#[derive(Debug)]
#[repr(C, align(8))]
pub struct Globals {
    pub frame: u32,
    pub _pad: u32,
    pub reso: [f32; 2],
}

unsafe impl AsBytes for Globals {}

extern "C" fn draw(mut dele: TypedObj<DrawState>, _sel: Sel, view: MTKView::IPtr) {
    let state = dele.get_inner();
    if state.ender.should_end() {
        return;
    }
    let frame = state.draw_receiver.get_fresh();

    let phase = state.frame % 100;
    let color_phase = 0.01 * phase as f64;

    let globals = Globals {
        frame: state.frame as u32,
        _pad: 0,
        reso: [
            state.config.resolution.0 as f32,
            state.config.resolution.1 as f32,
        ],
    };

    let pass_desc = view.current_rendpass_desc().or_(die!("rendpass_desc"));
    pass_desc
        .color_attach()
        .at(0)
        .set_clear_color(MTLClearColor::new(color_phase, 0.0, 0.0, 1.0));
    let cmd_buf = state.cmd_queue.cmd_buf().or_(die!("cmd_buf"));

    let rencoder = cmd_buf.rencoder_with_desc(pass_desc).or_(die!("rencoder"));
    rencoder.set_rend_pl_state(state.rend_pl_state);
    for cmd in frame.cmds {
        match cmd {
            &Cmd::DrawSQuads { pos, .. } => {
                rencoder.set_vtex_buf(state.quad_vtex_buf, 0, 0);
                rencoder.set_vtex_bytes(&globals, 1);
                rencoder.set_vtex_bytes(pos, 2);
                rencoder.set_vtex_buf(state.dbg_buf, 0, 3);
                rencoder.set_frag_tex(state.quad_tex, 0);
                rencoder.set_frag_buf(state.quad_pal_buf, 0, 1);
                rencoder.draw_primitives(MTLPrimitiveType::TriangleStrip, 0, 4, pos.len());
            }
        }
    }
    rencoder.end();

    let drawable = view.current_drawable().or_(die!("drawable"));
    cmd_buf.present_drawable(drawable);
    cmd_buf.commit();
    cmd_buf.wait_completion();

    state.frame += 1;
}

extern "C" fn size_change(
    _slf: TypedObj<DrawState>,
    _sel: Sel,
    _view: MTKView::IPtr,
    size: CGSize,
) {
    eprintln!("size change called?! {size:?}");
}

impl<'l> DrawState<'l> {
    pub fn init_delegate_cls() -> TypedCls<DrawState<'l>, MTKViewDelegate::PPtr> {
        let cls = TypedCls::make_class(c"MTKViewDelegateWithDrawState").or_(die!("UNREACHABLE"));
        MTKViewDelegate::PPtr::implement(&cls, draw, size_change);
        cls
    }

    pub fn new(
        device: MTLDevice::PPtr,
        pixel_fmt: MTLPixelFormat,
        draw_receiver: DrawReceiver<'l>,
        config: &'l Config,
        ender: &'l Ender,
    ) -> Self {
        let cmd_queue = device
            .new_cmd_queue()
            .or_(die!("new_cmd_queue: Failed to create command queue"));

        let vtex = [
            Vertex::new([-4.0, 4.0]),
            Vertex::new([-4.0, -4.0]),
            Vertex::new([4.0, 4.0]),
            Vertex::new([4.0, -4.0]),
        ];

        let quad_vtex_buf = device
            .new_buf(vtex.as_slice(), MTLResourceOptions::DEFAULT)
            .or_(die!("quad_vtex_buf: Failed to create vertex buffer"));

        let dbg = vec![
            VertexOut {
                color: [0.0, 0.0, 0.0, 0.0],
                pos: [0.0, 0.0, 0.0, 0.0]
            };
            1024
        ];
        let dbg_buf = device
            .new_buf(dbg.as_slice(), MTLResourceOptions::DEFAULT)
            .or_(die!("dbg_buf: Failed to create vertex buffer"));

        let pl_desc = MTLRenderPipelineDescriptor::IPtr::new();

        let options = MTLCompileOptions::IPtr::new();

        let lib = match device.new_lib_with_url(NSUrl::IPtr::new(c"target/shaders.metallib")) {
            Ok(lib) => lib,
            Err(_) => {
                eprintln!("Couldn't find precompiled shaders, compiling from source...");
                let mut source = std::fs::read_to_string("bang_rt/src/shaders.metal")
                    .or_(die!("Failed to read shader source"));
                source.push('\0');
                let source = NSString::IPtr::new(
                    &CString::from_vec_with_nul(source.into_bytes()).expect("UNREACHABLE"),
                );
                let lib = device
                    .new_lib_from_source(source, options)
                    .or_(die!("failed to create library"));
                eprintln!("Compiled!");
                lib
            }
        };

        pl_desc.set_vtex_fn(lib.new_fn(c"vertexShader"));
        pl_desc.set_frag_fn(lib.new_fn(c"fragmentShader"));
        let attach = pl_desc.color_attach().at(0);

        attach.set_pixel_fmt(pixel_fmt);

        let vtex_desc = MTLVertexDescriptor::IPtr::new();
        let attr_0 = vtex_desc.attributes().at(0);
        attr_0.set_format(MTLVertexFormat::Float2);
        attr_0.set_offset(0);
        attr_0.set_buffer_index(0);
        let layout_0 = vtex_desc.layouts().at(0);
        layout_0.set_stride(size_of::<Vertex>() as NSUInteger);
        pl_desc.set_vtex_desc(vtex_desc);

        let tex_desc = MTLTextureDescriptor::IPtr::new_2d(MTLPixelFormat::R8Uint, 8, 8);
        let quad_tex = device
            .new_tex(tex_desc)
            .or_(die!("quad_tex: Failed to create texture"));

        let region = MTLRegion {
            origin: MTLOrigin { x: 0, y: 0, z: 0 },
            size: MTLSize {
                width: 8,
                height: 8,
                depth: 1,
            },
        };

        let r: u8 = 0;
        let g = 1;
        let b = 2;

        quad_tex.replace(
            region,
            [
                [r, g, b, r, g, b, r, g],
                [g, b, r, g, b, r, g, b],
                [b, r, g, b, r, g, b, r],
                [r, g, b, r, g, b, r, g],
                [g, b, r, g, b, r, g, b],
                [b, r, g, b, r, g, b, r],
                [r, g, b, r, g, b, r, g],
                [g, b, r, g, b, r, g, b],
            ]
            .as_slice(),
            8,
        );

        let pal = [
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ];

        let quad_pal_buf = device
            .new_buf(pal.as_slice(), MTLResourceOptions::DEFAULT)
            .or_(die!("quad_pal_buf: Failed to create buffer"));

        let rend_pl_state = device.new_rend_pl_state(pl_desc).or_(die!(
            "new_rend_pl_state: Failed to create render pipeline state"
        ));

        Self {
            draw_receiver,
            cmd_queue,
            quad_vtex_buf,
            quad_tex,
            quad_pal_buf,
            dbg_buf,
            rend_pl_state,
            frame: 0,
            config,
            ender,
        }
    }
}

// The alignment is required because on GPU, we are using 2xf32 SIMD vectors
#[repr(C, align(8))]
struct Vertex {
    pos: [f32; 2],
}

#[derive(Debug, Clone)]
#[repr(C, align(16))]
struct VertexOut {
    color: [f32; 4],
    pos: [f32; 4],
}

impl Vertex {
    fn new(pos: [f32; 2]) -> Self {
        Vertex { pos }
    }
}

unsafe impl AsBytes for Vertex {}
unsafe impl AsBytes for VertexOut {}
