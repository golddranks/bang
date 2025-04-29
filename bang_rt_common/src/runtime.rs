use std::thread;

use crate::{
    alloc::{SharedAllocState, new_alloc_pair},
    draw::{DrawReceiver, SharedDrawState, new_draw_pair},
    input::{InputGatherer, SharedInputState, new_input_pair},
    load::{FrameLogic, get_frame_logic},
    logic_loop,
};

pub trait Runtime {
    type Window<'a>;
    fn init_rt();
    fn init_win<'l>(
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
    ) -> Self::Window<'l>;
    fn run(win: &mut Self::Window<'_>);
}

pub fn start_dynamic<'l, 's, RT: Runtime>(libname: &'s str) {
    let frame_logic = get_frame_logic(libname);
    start_rt::<RT>(frame_logic);
}

pub fn start_static<'l, RT: Runtime>(frame_logic: impl FrameLogic<'l>) {
    start_rt::<RT>(frame_logic);
}

pub fn start_rt<'l, RT: Runtime>(frame_logic: impl FrameLogic<'l>) {
    RT::init_rt();

    let mut shared_input_state = SharedInputState::default();
    let (input_gatherer, input_consumer) = new_input_pair(&mut shared_input_state);
    let mut shared_alloc_state = SharedAllocState::default();
    let (alloc_manager, mut alloc_retirer) = new_alloc_pair(&mut shared_alloc_state);
    let mut shared_draw_state = SharedDrawState::default();
    let (draw_sender, draw_receiver) = new_draw_pair(&mut shared_draw_state, &mut alloc_retirer);

    let mut window = RT::init_win(input_gatherer, draw_receiver);

    thread::scope(|s| {
        s.spawn(|| logic_loop::run(frame_logic, input_consumer, draw_sender, alloc_manager));
        RT::run(&mut window); // Runs in main thread because of possible platform thread limitations
    });

    println!("Bye!");
}
