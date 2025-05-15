use std::{
    any::Any,
    ffi::CStr,
    panic::{AssertUnwindSafe, catch_unwind},
    thread,
};

use bang_core::{
    Config,
    ffi::{Logic, LogicInitReturn, RtCtx, SendableErasedPtr},
};

use crate::{
    alloc::{SharedAllocState, make_alloc_tools},
    die,
    draw::{DrawReceiver, SharedDrawState, make_draw_tools},
    end::Ender,
    error::OrDie,
    input::{InputGatherer, SharedInputState, make_input_tools},
    load::dyn_load_logic,
    logic_loop::{self, RunArgs},
};

pub trait Runtime {
    type Window<'a>;
    fn init_rt(&self);
    fn init_win<'l>(
        &self,
        rt_ctx: &mut RtCtx,
        input_gatherer: InputGatherer<'l>,
        draw_receiver: DrawReceiver<'l>,
        ender: &'l Ender,
        config: &'l Config,
    ) -> Self::Window<'l>;
    fn run(win: &mut Self::Window<'_>);
    fn notify_end(ender: &Ender);
    fn new_ctx(&self) -> RtCtx;
}

pub fn start_dynamic<RT: Runtime>(rt: RT, lib: &CStr) {
    let logic = dyn_load_logic(lib);
    start_rt(rt, logic);
}

pub fn start_static<RT: Runtime>(rt: RT, logic: impl Logic) {
    start_rt(rt, logic);
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

pub fn start_rt<RT: Runtime>(rt: RT, logic: impl Logic) {
    rt.init_rt();

    let mut shared_alloc_state = SharedAllocState::default();
    let (mut alloc_manager, mut alloc_retirer, alloc_cleanup) =
        make_alloc_tools(&mut shared_alloc_state);

    let mut rt_ctx = rt.new_ctx();
    let mut mem = alloc_manager.get_alloc();
    let LogicInitReturn {
        logic_state,
        config,
    } = logic.init_raw(&mut mem, &mut rt_ctx);
    let seq = mem.alloc_seq;
    alloc_manager.retire_single(seq);

    let mut shared_input_state = SharedInputState::default();
    let (input_gatherer, input_consumer) = make_input_tools(&mut shared_input_state);
    let mut shared_draw_state = SharedDrawState::default();
    let (draw_sender, draw_receiver) = make_draw_tools(&mut shared_draw_state, &mut alloc_retirer);
    let ender = Ender::new(RT::notify_end);
    let mut window = rt.init_win(&mut rt_ctx, input_gatherer, draw_receiver, &ender, &config);

    let ender = &ender;
    let mut logic_err = None;
    let mut rt_err = None;
    let moved_logic_err = &mut logic_err;

    let args = RunArgs {
        logic,
        rt_ctx: &mut rt_ctx,
        state: SendableErasedPtr(logic_state),
        input_consumer,
        sender: draw_sender,
        alloc_manager,
        ender,
        config: &config,
    };

    thread::scope(|s| {
        thread::Builder::new()
            .name("logic_loop".to_owned())
            .spawn_scoped(s, move || {
                catch_unwind(AssertUnwindSafe(|| logic_loop::run(args)))
                    .unwrap_or_else(|err| *moved_logic_err = Some(err));
                ender.soft_quit();
            })
            .or_(die!("Unable to create thread"));

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
pub(crate) mod tests {
    use std::{ops::Not, ptr::null_mut, time::Instant};

    use arena::Id;
    use bang_core::{
        alloc::Mem,
        ffi::{RtKind, Tex},
        input::{Key, KeyState},
    };

    use test_normal_dylib::TestLogic as NormalTestLogic;
    use test_panic_dylib::TestLogic as PanicTestLogic;

    use super::*;

    #[derive(Default)]
    struct TestRT {
        crash: bool,
        synchro_crash: bool,
    }

    pub fn load_textures<'f>(_: &mut RtCtx, _: &[&str], _: &mut Mem<'f>) -> &'f [Id<Tex>] {
        unimplemented!()
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
            _: &mut RtCtx,
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

        fn new_ctx(&self) -> RtCtx {
            RtCtx {
                frame: 0,
                rt_kind: RtKind::Test,
                load_textures_ptr: load_textures,
                rt_state: SendableErasedPtr(null_mut()),
            }
        }
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_runtime_dynamic() {
        let rt = TestRT::default();
        start_dynamic(rt, c"../target/tests/libtest_normal_dylib.dylib");
    }

    #[test]
    fn test_runtime_inline() {
        let rt = TestRT::default();
        start_static(rt, NormalTestLogic);
    }

    #[test]
    #[should_panic(expected = "Logic loop panicked")]
    fn test_logic_panic() {
        let rt = TestRT::default();
        start_rt(rt, PanicTestLogic);
    }

    #[test]
    #[should_panic(expected = "Runtime loop panicked")]
    fn test_rt_panic() {
        let rt = TestRT {
            crash: true,
            synchro_crash: false,
        };
        start_rt(rt, NormalTestLogic);
    }

    #[test]
    #[should_panic(expected = "Runtime and logic loops both panicked")]
    fn test_rt_logic_panic() {
        let rt = TestRT {
            crash: false,
            synchro_crash: true,
        };
        start_rt(rt, PanicTestLogic);
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
