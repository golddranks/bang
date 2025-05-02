# Run and test targets

macos: target/debug/libdemo_impl.dylib bang_rt_macos/src/*.rs target/shaders.metallib
	cargo run --bin macos_runner target/debug/libdemo_impl.dylib

tui: target/debug/libdemo_impl.dylib bang_rt_tui/src/*.rs
	cargo run --bin tui_runner target/debug/libdemo_impl.dylib

demo: target/shaders.metallib
	cargo run --bin demo_dist

miri:
	cargo miri test

coverage: tarpaulin-report.html


# Build targets

target/debug/libdemo_impl.dylib: demo/demo_impl/src/*.rs
	cargo build -p demo_impl

target/shaders.metallib: bang_rt_macos/src/shaders.metal
	mkdir -p target
	xcrun -sdk macosx metal -o target/shaders.ir -c bang_rt_macos/src/shaders.metal
	xcrun -sdk macosx metallib -o target/shaders.metallib target/shaders.ir

target/tests/libtest_normal_dylib.dylib: bang_rt_common/tests/test_normal_dylib/src/*.rs
	cargo build -p test_normal_dylib --features export
	mkdir -p target/tests
	cp target/debug/libtest_normal_dylib.dylib target/tests/libtest_normal_dylib.dylib

target/tests/libtest_panic_dylib.dylib: bang_rt_common/tests/test_panic_dylib/src/*.rs
	cargo build -p test_panic_dylib --features export
	mkdir -p target/tests
	cp target/debug/libtest_panic_dylib.dylib target/tests/libtest_panic_dylib.dylib

target/tests/libtest_symbol_missing_dylib.dylib: bang_rt_common/tests/test_symbol_missing_dylib/src/*.rs
	cargo build -p test_symbol_missing_dylib
	mkdir -p target/tests
	cp target/debug/libtest_symbol_missing_dylib.dylib target/tests/libtest_symbol_missing_dylib.dylib

tarpaulin-report.html:	bang_rt_common/src/*.rs \
						bang_core/src/*.rs \
						target/tests/libtest_normal_dylib.dylib \
						target/tests/libtest_panic_dylib.dylib \
						target/tests/libtest_symbol_missing_dylib.dylib
	cargo tarpaulin -p bang_rt_common -p bang_core --lib -o html
