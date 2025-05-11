# Run and test targets

macos: target/debug/libdemo_impl.dylib macos_rt assets
	cargo run --bin macos_runner target/debug/libdemo_impl.dylib

tui: target/debug/libdemo_impl.dylib tui_rt assets
	cargo run --bin tui_runner target/debug/libdemo_impl.dylib

static_demo: macos_rt assets
	cargo run --bin demo_dist

miri:
	cargo miri test

coverage: tarpaulin-report.html

assets: assets/paltex/*.paltex


# Build targets

core: bang_core/src/*.rs libs/*/src/*.rs
	cargo build -p bang_core

macos_rt: runtimes/bang_rt_macos/src/*.rs core target/shaders.metallib
	cargo build -p bang_rt_macos

tui_rt: runtimes/bang_rt_tui/src/*.rs core
	cargo build -p bang_rt_tui

target/debug/libdemo_impl.dylib: core
	cargo build -p demo_impl

target/shaders.metallib: runtimes/bang_rt_macos/src/shaders.metal
	mkdir -p target
	xcrun -sdk macosx metal -o target/shaders.ir -c $<
	xcrun -sdk macosx metallib -o $@ target/shaders.ir

target/tests/lib%.dylib: bang_rt_common/tests/%/src/*.rs
	cargo build -p $* --features export
	mkdir -p target/tests
	cp target/debug/lib$*.dylib $@

tarpaulin-report.html:	bang_rt_common/src/*.rs \
						bang_core/src/*.rs \
						libs/*/src/*.rs \
						target/tests/*.dylib
	cargo tarpaulin -p bang_rt_common -p bang_core -p paltex -p arena --lib -o html

assets/paltex/%.paltex: libs/paltex/src/*.rs tools/png2paltex/src/*.rs assets/png/%.png
	cd assets/paltex && cargo run --bin png2paltex -- ../png
