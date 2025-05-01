mod draw;
mod win;

use bang_rt_common::{draw::DrawReceiver, end::Ender, input::InputGatherer, runtime::Runtime};
use win::Window;

pub struct TuiRT;

impl Runtime for TuiRT {
    type Window<'a> = Window<'a>;

    fn init_rt(&self) {}

    fn init_win<'l>(
        &self,
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &'l Ender,
    ) -> Self::Window<'l> {
        Window::init(input_gatherer, draw_receiver, ender)
    }

    fn run(win: &mut Self::Window<'_>) {
        win.run();
    }

    fn notify_end(_: &Ender) {}
}
