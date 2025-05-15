# TODO

## Currently working on

- Resource loading (textures)

## Short term

- Entity ID system
- Hot reloading
  - File watching
  - Function pointer swap
  - Make Entity ID system work with hot reloading
- Better input handling
- Fixed point math basics
- Position + velocity components
- Simple collision system

## Long term

- Fixed point math advanced
- Audio fundamentals
- WebAssembly & WebGPU

## Very long term

- Vulkan, WinAPI, Wayland

# Wants for Rust

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
