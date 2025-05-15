use std::ffi::CString;

use bang_core::{
    Config,
    alloc::{Id, Mem},
    draw::{AsBytes, Cmd, ScreenPos},
    ffi::{RtCtx, Tex},
};
use bang_rt_common::{
    die,
    draw::{Color, DrawReceiver, PalTex},
    end::Ender,
    error::OrDie,
};

use crate::{
    RtState,
    objc::{
        NSString, NSUInteger, Sel, TypedCls, TypedObj,
        wrappers::{
            CGSize, MTKView, MTKViewDelegate, MTLBlendFactor, MTLBuffer, MTLClearColor,
            MTLCommandQueue, MTLCompileOptions, MTLDevice, MTLOrigin, MTLPixelFormat,
            MTLPrimitiveType, MTLRegion, MTLRenderCommandEncoder, MTLRenderPipelineDescriptor,
            MTLRenderPipelineState, MTLResourceOptions, MTLSize, MTLTexture, MTLTextureDescriptor,
            MTLTextureType, MTLVertexDescriptor, MTLVertexFormat, NSUrl,
        },
    },
};

#[derive(Debug)]
pub struct DrawState<'l> {
    draw_receiver: DrawReceiver<'l>,
    cmd_queue: MTLCommandQueue::PPtr,
    quad_vtex_buf: MTLBuffer::PPtr,
    bound_paltex: Vec<BoundPalTex>,
    rend_pl_state: MTLRenderPipelineState::PPtr,
    frame: usize,
    config: &'l Config,
    ender: &'l Ender,
}

#[derive(Debug)]
#[repr(C, align(8))]
pub struct Globals {
    pub frame: u32,
    pub quad_size: [u16; 2],
    pub reso: [f32; 2],
}

unsafe impl AsBytes for Globals {}

extern "C" fn draw(mut dele: TypedObj<DrawState>, _sel: Sel, view: MTKView::IPtr) {
    let state = dele.get_inner();
    if state.ender.should_end() {
        return;
    }
    let frame = state.draw_receiver.get_fresh();

    let mut globals = Globals {
        frame: state.frame as u32,
        quad_size: [8, 8],
        reso: [
            state.config.resolution.0 as f32,
            state.config.resolution.1 as f32,
        ],
    };

    let pass_desc = view.current_rendpass_desc().or_(die!("rendpass_desc"));
    pass_desc
        .color_attach()
        .at(0)
        .set_clear_color(MTLClearColor::new(0.0, 0.0, 0.0, 1.0));
    let cmd_buf = state.cmd_queue.cmd_buf().or_(die!("cmd_buf"));

    let rencoder = cmd_buf.rencoder_with_desc(pass_desc).or_(die!("rencoder"));
    rencoder.set_rend_pl_state(state.rend_pl_state);
    for cmd in frame.cmds {
        match cmd {
            &Cmd::DrawDummies { pos } => {
                let bound_paltex = &state.bound_paltex[0];
                globals.quad_size = bound_paltex.quad_size;
                draw_squad(&rencoder, state.quad_vtex_buf, &globals, pos, bound_paltex);
            }
            &Cmd::DrawSQuads { pos, tex } => {
                let bound_paltex = &state.bound_paltex[tex.idx()];
                globals.quad_size = bound_paltex.quad_size;
                draw_squad(&rencoder, state.quad_vtex_buf, &globals, pos, bound_paltex);
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
        mut device: MTLDevice::PPtr,
        pixel_fmt: MTLPixelFormat,
        draw_receiver: DrawReceiver<'l>,
        config: &'l Config,
        ender: &'l Ender,
    ) -> Self {
        let cmd_queue = device
            .new_cmd_queue()
            .or_(die!("new_cmd_queue: Failed to create command queue"));

        let vtex = [
            Vertex::new([0.0, 1.0]),
            Vertex::new([0.0, 0.0]),
            Vertex::new([1.0, 1.0]),
            Vertex::new([1.0, 0.0]),
        ];

        let quad_vtex_buf = device
            .new_buf(vtex.as_slice(), MTLResourceOptions::DEFAULT)
            .or_(die!("quad_vtex_buf: Failed to create vertex buffer"));

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

        attach.set_blend_enabled(true);
        attach.set_pixel_fmt(pixel_fmt);
        attach.set_source_rgb_blend_factor(MTLBlendFactor::SourceAlpha);
        attach.set_source_alpha_blend_factor(MTLBlendFactor::SourceAlpha);
        attach.set_dest_rgb_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);
        attach.set_dest_alpha_blend_factor(MTLBlendFactor::OneMinusSourceAlpha);

        let vtex_desc = MTLVertexDescriptor::IPtr::new();
        let attr_0 = vtex_desc.attributes().at(0);
        attr_0.set_format(MTLVertexFormat::Float2);
        attr_0.set_offset(0);
        attr_0.set_buffer_index(0);
        let layout_0 = vtex_desc.layouts().at(0);
        layout_0.set_stride(size_of::<Vertex>() as NSUInteger);
        pl_desc.set_vtex_desc(vtex_desc);

        let transp = Color::from_rgba_f32([0.0, 0.0, 0.0, 0.0]);
        let red = Color::from_rgba_f32([1.0, 0.0, 0.0, 1.0]);
        let halfred = Color::from_rgba_f32([1.0, 0.0, 0.0, 0.5]);
        let blue = Color::from_rgba_f32([0.0, 0.0, 1.0, 1.0]);

        let smile = PalTex::from_ascii_map(
            &[(b' ', transp), (b'O', red), (b'I', halfred), (b'_', blue)],
            &[
                *b"  ____  ",
                *b" ______ ",
                *b"_O____O_",
                *b"___II___",
                *b"O______O",
                *b"_O____O_",
                *b" _OOOO_ ",
                *b"  ____  ",
            ],
        );
        let smile = BoundPalTex::new(&smile, &mut device);

        let bubu =
            PalTex::from_encoded(&std::fs::read("assets/paltex/bubu.paltex").unwrap()).unwrap();
        let bubu = BoundPalTex::new(&bubu, &mut device);

        let toge =
            PalTex::from_encoded(&std::fs::read("assets/paltex/toge.paltex").unwrap()).unwrap();
        let toge = BoundPalTex::new(&toge, &mut device);

        let lima =
            PalTex::from_encoded(&std::fs::read("assets/paltex/lima.paltex").unwrap()).unwrap();
        let lima = BoundPalTex::new(&lima, &mut device);

        let rend_pl_state = device.new_rend_pl_state(pl_desc).or_(die!(
            "new_rend_pl_state: Failed to create render pipeline state"
        ));

        Self {
            draw_receiver,
            cmd_queue,
            quad_vtex_buf,
            bound_paltex: vec![smile, bubu, toge, lima],
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

impl Vertex {
    fn new(pos: [f32; 2]) -> Self {
        Vertex { pos }
    }
}

unsafe impl AsBytes for Vertex {}

#[derive(Debug)]
pub struct BoundPalTex {
    quad_size: [u16; 2],
    pal: MTLTexture::PPtr,
    tex: MTLTexture::PPtr,
}

impl BoundPalTex {
    fn new(paltex: &PalTex, device: &mut MTLDevice::PPtr) -> Self {
        let tex_desc = MTLTextureDescriptor::IPtr::new_2d(
            MTLPixelFormat::R8Uint,
            paltex.width as usize,
            paltex.height as usize,
        );

        let tex = device
            .new_tex(tex_desc)
            .or_(die!("tex: Failed to create texture"));

        tex.replace(
            MTLRegion {
                origin: MTLOrigin { x: 0, y: 0, z: 0 },
                size: MTLSize {
                    width: paltex.width as u64,
                    height: paltex.height as u64,
                    depth: 1,
                },
            },
            paltex.data.as_slice(),
            paltex.width as usize * size_of::<u8>(),
        );

        let pal_desc = MTLTextureDescriptor::IPtr::new();

        pal_desc.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        pal_desc.set_texture_type(MTLTextureType::T1D);
        pal_desc.set_width(paltex.palette.len() as u64);

        let pal = device
            .new_tex(pal_desc)
            .or_(die!("pal: Failed to create palette texture"));

        pal.replace(
            MTLRegion {
                origin: MTLOrigin { x: 0, y: 0, z: 0 },
                size: MTLSize {
                    width: paltex.palette.len() as u64,
                    height: 1,
                    depth: 1,
                },
            },
            paltex.palette.as_slice(),
            0,
        );

        Self {
            quad_size: [paltex.width as u16, paltex.height as u16],
            pal,
            tex,
        }
    }

    fn bind_frag(&self, rencoder: &MTLRenderCommandEncoder::PPtr) {
        rencoder.set_frag_tex(self.tex, 0);
        rencoder.set_frag_tex(self.pal, 1);
    }
}

fn draw_squad(
    rencoder: &MTLRenderCommandEncoder::PPtr,
    quad_vtex_buf: MTLBuffer::PPtr,
    globals: &Globals,
    pos: &[ScreenPos],
    pal: &BoundPalTex,
) {
    rencoder.set_vtex_buf(quad_vtex_buf, 0, 0);
    rencoder.set_vtex_bytes(globals, 1);
    rencoder.set_vtex_bytes(pos, 2);
    pal.bind_frag(rencoder);
    rencoder.draw_primitives(MTLPrimitiveType::TriangleStrip, 0, 4, pos.len());
}

pub fn load_textures<'f>(rt_ctx: &mut RtCtx, tex: &[&str], mem: &mut Mem<'f>) -> &'f [Id<Tex>] {
    let mut ids = mem.sink();
    let rt = RtState::unwrap_from(rt_ctx);
    for &t in tex {
        let tex = PalTex::from_encoded(&std::fs::read(t).unwrap()).unwrap();
        let bound_tex = BoundPalTex::new(&tex, &mut rt.device);
        ids.push(rt.textures.alloc_upcast(bound_tex));
    }
    ids.into_slice()
}
