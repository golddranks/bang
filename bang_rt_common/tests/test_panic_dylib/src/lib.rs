use bang_core::{
    Config,
    alloc::Mem,
    draw::DrawFrame,
    ffi::{Logic, RtCtx},
    input::InputState,
};

pub struct TestLogic;

impl Logic for TestLogic {
    type S = ();

    fn new() -> Self {
        TestLogic
    }

    fn init(&self, _: &mut Mem, _: &mut RtCtx) -> (Self::S, Config) {
        (
            (),
            Config {
                name: "Demo",
                resolution: (320, 200),
                logic_fps: 60,
                scale: 1,
            },
        )
    }

    fn update<'f>(
        &self,
        _: &mut Mem<'f>,
        _: &InputState,
        _: &mut RtCtx,
        _: &mut Self::S,
    ) -> DrawFrame<'f> {
        panic!("Oh, no!");
    }
}

#[cfg(feature = "export")]
bang_core::export_logic!(TestLogic);
