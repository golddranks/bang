shaders: bang_rt/src/shaders.metal
	xcrun -sdk macosx metal -o target/shaders.ir -c bang_rt/src/shaders.metal
	xcrun -sdk macosx metallib -o target/shaders.metallib target/shaders.ir

demo: demo_impl/src/*.rs
	cargo build -p demo_impl

run: shaders demo
	cargo run --bin runner demo_impl

demo_runner:
	cargo run --bin demo_runner

.PHONY: run
.PHONY: demo_runner
