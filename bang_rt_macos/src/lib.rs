mod draw;
mod objc;
mod timer;
mod win;

use bang_rt_common::Runtime;
pub use win::Window;

pub struct MacOSRT;

impl Runtime for MacOSRT {
    type Window<'a> = Window<'a>;

    fn init_rt() {
        objc::init_objc();
    }

    fn init_win<'l>(
        input_gatherer: bang_rt_common::input::InputGatherer<'l>,
        draw_receiver: bang_rt_common::draw::DrawReceiver<'l>,
    ) -> Self::Window<'l> {
        Window::init(input_gatherer, draw_receiver)
    }

    fn run(win: &mut Self::Window<'_>) {
        win.run();
    }
}
