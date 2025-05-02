mod draw;
mod objc;
mod timer;
mod win;

use bang_rt_common::{draw::DrawReceiver, end::Ender, input::InputGatherer, runtime::Runtime};

use win::Window;
pub struct MacOSRT;

impl Runtime for MacOSRT {
    type Window<'a> = Window<'a>;

    fn init_rt(&self) {
        objc::init_objc();
    }

    fn init_win<'l>(
        &self,
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &Ender,
    ) -> Self::Window<'l> {
        ender.install_global_signal_handler();
        Window::init(input_gatherer, draw_receiver)
    }

    fn run(win: &mut Self::Window<'_>) {
        win.run();
    }

    fn notify_end(ender: &Ender) {
        Window::notify_end(ender)
    }
}
