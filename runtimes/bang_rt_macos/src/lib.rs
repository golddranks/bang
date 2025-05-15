mod draw;
mod objc;
mod timer;
mod win;

use bang_core::{
    Config,
    alloc::Managed,
    ffi::{RtCtx, RtKind, SendableErasedPtr, Tex},
};
use bang_rt_common::{draw::DrawReceiver, end::Ender, input::InputGatherer, runtime::Runtime};

use draw::BoundPalTex;
use objc::wrappers::MTLDevice;
use win::Window;

pub struct MacOSRT;

impl Runtime for MacOSRT {
    type Window<'a> = Window<'a>;

    fn init_rt(&self) {
        objc::init_objc();
    }

    fn init_win<'l>(
        &self,
        rt_ctx: &mut RtCtx,
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &'l Ender,
        config: &'l Config,
    ) -> Self::Window<'l> {
        ender.install_global_signal_handler();
        Window::init(rt_ctx, input_gatherer, draw_receiver, config, ender)
    }

    fn run(win: &mut Self::Window<'_>) {
        win.run();
    }

    fn notify_end(ender: &Ender) {
        Window::notify_end(ender)
    }

    fn new_ctx(&self) -> RtCtx {
        let device = MTLDevice::PPtr::get_default();
        let rt_state = Box::new(RtState {
            device,
            textures: Managed::default(),
        });
        RtCtx {
            frame: 0,
            rt_kind: RtKind::MacOS,
            load_textures_ptr: draw::load_textures,
            rt_state: SendableErasedPtr::wrap(rt_state),
        }
    }
}

struct RtState {
    device: MTLDevice::PPtr,
    textures: Managed<BoundPalTex, Tex>,
}

impl RtState {
    fn unwrap_from(rt_ctx: &mut RtCtx) -> &mut RtState {
        unsafe { &mut *(rt_ctx.rt_state.0 as *mut RtState) }
    }
}
