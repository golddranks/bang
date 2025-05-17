# BANG!

**NOTE:** This is a work in progress. I haven't released even the first version yet! Proceed with caution!

Bang is a 2D game engine, written in Rust, from scratch.
(Currently no external dependencies other than Rust stdlib and the OS platform APIs,
and the only available runtime is macOS / Metal. Other runtimes are planned.)

It's meant for quick prototyping simple games, for game jams and small projects.

The values I try to cherish as I'm developing Bang (in order):

1. It must spark joy for me; it's a side project, so it must stay interesting and fun to survive.
2. Easy to start up, quick to iterate, quick to test out game ideas; that's the main purpose of the project.
3. Robust and portable; if the game works on the developer's computer, it should trivially work on
   every supported platform. Showing off your creations is the next best thing to making them, right?
4. Performant; because why not. Software should be performant. I hate slow programs and I hate tools
   that seem polished but turn out to be turds of glitchy slowness at the moment you try to do anything
   serious with them.

These values are my north star. They guide my decisions and help me prioritize features and improvements.
The concrete design they lead me to, isn't clear yet, however. Currently, my design looks like:

- Focus on building things from scratch. I'm curious and love learning and designing things from ground up.
  This is why I'm choosing to focus on writing basically everything myself, at the cost of the speed of development
  of the engine. I'm also a firm believer in craftsmanship in software. By understanding each nook and cranny;
  thinking, researching, and building code that is carefully thought of, I think I can reach eventually better outcome
  than whipping together a bunch of libraries with a lot of fluff. (Principaly value #1, but by the expected outcomes,
  also #2, #3 and #4)
- Focus on 2D. The engine is meant for simple games, created in swift creative bursts. 3D just seems like
  too much effort for the cost. Also, while I enjoy modern games, developing those for hobby sounds...
  almost impossible. Just coming by with all the 3D assets is too much for a quick game.
  I love indies for the creativity and the simplicity! (Values #1 and #2)
- To use Rust. I think Rust is the only viable programming language that aligns with my goals in year 2025.
  Zig almost makes it, but I'm too unfamiliar with it (Value #1), and it apparently changes too often as it
  isn't 1.0 yet (Values #1, #3). Languages with heavy runtimes or GC are out of question. (Values #3 and #4)
  C and C++ are out of question. (Values #1 and #3)
- To separate logic and rendering loops. I want the fixed-time logic loop to be the default. Frame rates
  of the gamers' screens be damned! I want to be free of considerations like: "does this feel totally different
  for people with 30fps, 60fps and 120fps", and I want to free the developer of the game from having to think
  those kinds of details. Delta-time shall be a thing, but let it be a constant. (Value #3, but in a way, also #2)
- To use fixed-point arithmetic for in-game-world coordinates and calculations. This might feel weird from the
  value #4 perspective, as floating point arithmetic is fast and the standard in games, but...
  because value #3 is before #4, I want to use the option that is the deterministic and more correct.
  Specifically, I want to respect the "translational symmetry" of the game world; with floating points,
  a result of a simulation is dependent on _where_ the simulation happens within the game world. I don't like that.
- To provide an easy, seeded, deterministic PRNG, instead of users having to reach for platform randomness.
  Seems like a no-brainer. (Values #2 and #3)
- To provide very performant and easy-to-use allocators, both for frame lifetimes and managed, dynamic lifetimes.
  Managed means that we are going to use IDs instead of pointers. That removes a big footgun, and because
  deallocating is going to to be manual and deterministic, lifetimes are also clear. (Values #2, #3 and #4)

## TODO

### Currently working on

- Resource loading (textures)
- Consider AllocGuard that is created anew in fresh_frame, instead of transmuting
- Make alloc_seq a shared atomic between all allocators
- Make managed allocator multi-threaded

### Short term

- Entity ID system
- Hot reloading
  - File watching
  - Function pointer swap
  - Make Entity ID system work with hot reloading
- Better input handling
- Fixed point math basics
- Position + velocity components
- Simple collision system

### Long term

- Fixed point math advanced
- Audio fundamentals
- WebAssembly & WebGPU

### Very long term

- Vulkan, WinAPI, Wayland

## Wants for Rust

- Stable ABI
  - At least stable slice FFI.
    - Guarantee slice representation: https://github.com/rust-lang/rfcs/pull/3775
    - crABI: https://github.com/rust-lang/rfcs/pull/3470
- Safer dynamic linking (mangled + statically checked)
  - #[export] (dynamically linked crates): https://github.com/rust-lang/rfcs/pull/3435
- Nicer macro_rules
  - Referring items inside macros are unintuitive (one would like to have them lexical scope-based but they are not)
  - Macros as "items" are namespaced weirdly (#[macro_use], #[macro_export], no pub etc.)
  - Lack of expressiveness w.r.t recursion and counting (some of the stuff is about to get fixed with metavar expressions such as $count)
  - Lack of concat_idents (could be implemented with a metavar expression)
  - Lack of niceties for some common but complicated language syntax (will get better with some more sophisticated fragment specifiers)
  - Some language features could be improved to be more useful with macros (associated statics, re-opening mod blocks)
  - No suffix macros
- Unsafe lifetime binders
  - https://hackmd.io/@compiler-errors/HkXwoBPaR
- Miri-friendly APIs for memory allocation shenanigans
  - https://github.com/rust-lang/rust/issues/129090
  - https://github.com/rust-lang/rust/issues/74265
