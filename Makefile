shaders: bang_rt/src/shaders.metal
	xcrun -sdk macosx metal -o target/shaders.ir -c bang_rt/src/shaders.metal
	xcrun -sdk macosx metallib -o target/shaders.metallib target/shaders.ir

demo: demo/src/*.rs
	cargo build -p demo

run: shaders demo
	cargo run -- demo

.PHONY: run
