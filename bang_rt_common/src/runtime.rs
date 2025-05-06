use std::{
    any::Any,
    ffi::CStr,
    panic::{AssertUnwindSafe, catch_unwind},
    thread,
};

use bang_core::Config;

use crate::{
    alloc::{SharedAllocState, make_alloc_tools},
    draw::{DrawReceiver, SharedDrawState, make_draw_tools},
    end::Ender,
    input::{InputGatherer, SharedInputState, make_input_tools},
    load::{FrameLogic, get_symbols},
    logic_loop,
};

pub trait Runtime {
    type Window<'a>;
    fn init_rt(&self);
    fn init_win<'l>(
        &self,
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &'l Ender,
        config: &'l Config,
    ) -> Self::Window<'l>;
    fn run(win: &mut Self::Window<'_>);
    fn notify_end(ender: &Ender);
}

pub fn start_dynamic<RT: Runtime>(rt: RT, lib: &CStr) {
    let (frame_logic, config) = get_symbols(lib);
    start_rt(rt, frame_logic, &config);
}

pub fn start_static<'l, RT: Runtime>(
    rt: RT,
    frame_logic: impl FrameLogic<'l>,
    config: &'static Config,
) {
    start_rt(rt, frame_logic, config);
}

fn downcast(msg: &Box<dyn Any + Send>) -> &str {
    match msg.downcast_ref::<&'static str>() {
        Some(s) => s,
        None => match msg.downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "",
        },
    }
}

pub fn start_rt<'l, RT: Runtime>(rt: RT, frame_logic: impl FrameLogic<'l>, config: &'l Config) {
    rt.init_rt();

    let mut shared_input_state = SharedInputState::default();
    let (input_gatherer, input_consumer) = make_input_tools(&mut shared_input_state);
    let mut shared_alloc_state = SharedAllocState::default();
    let (alloc_manager, mut alloc_retirer, alloc_cleanup) =
        make_alloc_tools(&mut shared_alloc_state);
    let mut shared_draw_state = SharedDrawState::default();
    let (draw_sender, draw_receiver) = make_draw_tools(&mut shared_draw_state, &mut alloc_retirer);
    let ender = Ender::new(RT::notify_end);
    let mut window = rt.init_win(input_gatherer, draw_receiver, &ender, config);

    let mut logic_err = None;
    let mut rt_err = None;

    thread::scope(|s| {
        s.spawn(|| {
            catch_unwind(AssertUnwindSafe(|| {
                logic_loop::run(
                    frame_logic,
                    input_consumer,
                    draw_sender,
                    alloc_manager,
                    &ender,
                    config,
                )
            }))
            .unwrap_or_else(|err| logic_err = Some(err));
            ender.soft_quit();
        });

        // Runs in main thread because of possible platform API thread safety limitations
        catch_unwind(AssertUnwindSafe(|| RT::run(&mut window)))
            .unwrap_or_else(|err| rt_err = Some(err));
        // Important to drop DrawReceiver to free frame allocations;
        // otherwise, logic_loop's alloc_manager.wait_until_done() waits forever for cleanup
        alloc_cleanup.cleanup();
        ender.soft_quit();
    });

    match (rt_err, logic_err) {
        (Some(rt_err), Some(logic_err)) => {
            let rt_err = downcast(&rt_err);
            let logic_err = downcast(&logic_err);
            panic!("Runtime and logic loops both panicked: {rt_err}, {logic_err}");
        }
        (Some(rt_err), None) => {
            let rt_err = downcast(&rt_err);
            panic!("Runtime loop panicked: {rt_err}");
        }
        (None, Some(logic_err)) => {
            let logic_err = downcast(&logic_err);
            panic!("Logic loop panicked: {logic_err}");
        }
        (None, None) => {}
    }
    println!("Bye!");
}

#[cfg(test)]
mod tests {
    use std::{ops::Not, time::Instant};

    use bang_core::input::{Key, KeyState};

    use test_normal_dylib::test_frame_logic_normal;
    use test_panic_dylib::test_frame_logic_panicking;

    use crate::load::InlinedFrameLogic;

    use super::*;

    const TEST_CONFIG: Config = Config {
        name: "Test",
        resolution: (800, 600),
        logic_fps: 60,
        scale: 1,
    };

    #[derive(Default)]
    struct TestRT {
        crash: bool,
        synchro_crash: bool,
    }

    struct TestWindow<'l> {
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &'l Ender,
        crash: bool,
        synchro_crash: bool,
    }

    impl Runtime for TestRT {
        type Window<'l> = TestWindow<'l>;

        fn init_rt(&self) {}

        fn init_win<'l>(
            &self,
            input_gatherer: InputGatherer<'l>,
            draw_receiver: DrawReceiver<'l>,
            ender: &'l Ender,
            _config: &'l Config,
        ) -> Self::Window<'l> {
            TestWindow {
                input_gatherer,
                draw_receiver,
                ender,
                crash: self.crash,
                synchro_crash: self.synchro_crash,
            }
        }

        fn run(win: &mut Self::Window<'_>) {
            if win.crash {
                panic!("TestRT crashed!");
            }
            if win.synchro_crash {
                while win.ender.should_end().not() {} // Busy wait for the other thread to crash
                panic!("TestRT crashed synchronously with the logic loop!");
            }
            assert!(win.draw_receiver.has_fresh().not());
            win.input_gatherer
                .update(Key::Space, KeyState::Pressed, Instant::now());

            while win.draw_receiver.get_fresh().alloc_seq < 1 {
                if win.ender.should_end() {
                    return;
                }
            }

            let fresh = win.draw_receiver.get_fresh();
            assert_eq!(fresh.alloc_seq, 1);
        }

        fn notify_end(_: &Ender) {}
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_runtime_dynamic() {
        let rt = TestRT::default();
        start_dynamic(rt, c"../target/tests/libtest_normal_dylib.dylib");
    }

    #[test]
    fn test_runtime_inline() {
        let fl = InlinedFrameLogic::new(test_frame_logic_normal);
        let rt = TestRT::default();
        start_static(rt, fl, &TEST_CONFIG);
    }

    #[test]
    #[should_panic(expected = "Logic loop panicked")]
    fn test_logic_panic() {
        let fl = InlinedFrameLogic::new(test_frame_logic_panicking);
        let rt = TestRT::default();
        start_rt(rt, fl, &TEST_CONFIG);
    }

    #[test]
    #[should_panic(expected = "Runtime loop panicked")]
    fn test_rt_panic() {
        let fl = InlinedFrameLogic::new(test_frame_logic_normal);
        let rt = TestRT {
            crash: true,
            synchro_crash: false,
        };
        start_rt(rt, fl, &TEST_CONFIG);
    }

    #[test]
    #[should_panic(expected = "Runtime and logic loops both panicked")]
    fn test_rt_logic_panic() {
        let fl = InlinedFrameLogic::new(test_frame_logic_panicking);
        let rt = TestRT {
            crash: false,
            synchro_crash: true,
        };
        start_rt(rt, fl, &TEST_CONFIG);
    }

    #[test]
    fn test_downcast() {
        let b = Box::new(()) as Box<dyn Any + Send + 'static>;
        assert_eq!(downcast(&b), "");

        let b = Box::new("hello") as Box<dyn Any + Send + 'static>;
        assert_eq!(downcast(&b), "hello");

        let b = Box::new(format!("world {a}", a = 1 + 1)) as Box<dyn Any + Send + 'static>;
        assert_eq!(downcast(&b), "world 2");
    }
}
