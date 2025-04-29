mod draw;
mod win;

use bang_rt_common::{draw::DrawReceiver, input::InputGatherer, runtime::Runtime};
use win::Window;

pub struct TuiRT;

impl Runtime for TuiRT {
    type Window<'a> = Window<'a>;

    fn init_rt() {}

    fn init_win<'l>(
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
    ) -> Self::Window<'l> {
        Window::init(input_gatherer, draw_receiver)
    }

    fn run(win: &mut Self::Window<'_>) {
        win.run();
    }
}
