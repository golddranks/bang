# Sources

PNG_ASSETS := $(wildcard assets/png/*.png)
PALTEX_ASSETS := $(patsubst assets/png/%.png,assets/paltex/%.paltex,$(PNG_ASSETS))

TESTS := $(notdir $(wildcard bang_rt_common/tests/*))
TEST_DYLIBS := $(addprefix target/tests/lib,$(addsuffix .dylib,$(TESTS)))

CORE_SRC := $(wildcard bang_core/src/*.rs) $(wildcard libs/*/src/*.rs)
MACOS_RT_SRC := $(wildcard runtimes/bang_rt_macos/src/*.rs)
TUI_RT_SRC := $(wildcard runtimes/bang_rt_tui/src/*.rs)

.PHONY: macos tui static_demo miri coverage assets test_dylib core macos_rt tui_rt clean

# Run and test targets

macos: target/debug/libdemo_main.dylib macos_rt assets
	cargo run --bin macos_runner target/debug/libdemo_main.dylib

tui: target/debug/libdemo_main.dylib tui_rt assets
	cargo run --bin tui_runner target/debug/libdemo_main.dylib

static_demo: macos_rt assets
	cargo run --bin demo_static

miri:
	cargo miri test

coverage: tarpaulin-report.html

assets: $(PALTEX_ASSETS)

test_dylib: $(TEST_DYLIBS)

core: $(CORE_SRC)
	cargo build -p bang_core

macos_rt: $(MACOS_RT_SRC) core target/shaders.metallib
	cargo build -p bang_rt_macos

tui_rt: $(TUI_RT_SRC) core
	cargo build -p bang_rt_tui

target/debug/libdemo_main.dylib: core
	cargo build -p demo_main

target/shaders.metallib: runtimes/bang_rt_macos/src/shaders.metal | target
	xcrun -sdk macosx metal -o target/shaders.ir -c $<
	xcrun -sdk macosx metallib -o $@ target/shaders.ir

target/tests/lib%.dylib: bang_rt_common/tests/%/src/*.rs | target/tests
	cargo build -p $*
	cp target/debug/lib$*.dylib $@

tarpaulin-report.html:	macos_rt \
						core \
						test_dylib
	cargo tarpaulin -p bang_rt_common -p bang_core -p paltex -p arena --lib -o html

assets/paltex/%.paltex: assets/png/%.png libs/paltex/src/*.rs tools/png2paltex/src/*.rs | assets/paltex
	cd assets/paltex && cargo run --bin png2paltex -- ../png/$*.png

target:
	mkdir -p target

target/tests:
	mkdir -p target/tests

assets/paltex:
	mkdir -p assets/paltex

clean:
	rm -rf target
	rm -f tarpaulin-report.html
	rm -f assets/paltex/*.paltex
