macos_runner: target/shaders.metallib target/debug/libdemo_impl.dylib
	cargo run --bin macos_runner demo_impl

demo_dist: target/shaders.metallib target/debug/libdemo_impl.dylib
	cargo run --bin demo_dist

target/shaders.metallib: bang_rt_macos/src/shaders.metal
	xcrun -sdk macosx metal -o target/shaders.ir -c bang_rt_macos/src/shaders.metal
	xcrun -sdk macosx metallib -o target/shaders.metallib target/shaders.ir

target/debug/libdemo_impl.dylib: demo_impl/src/*.rs
	cargo build -p demo_impl

tui_runner: target/shaders.metallib target/debug/libdemo_impl.dylib
	cargo run --bin tui_runner demo_impl

.PHONY: demo_dist
