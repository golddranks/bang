shaders: src/shaders.metal
	xcrun -sdk macosx metal -o target/shaders.ir -c src/shaders.metal
	xcrun -sdk macosx metallib -o target/shaders.metallib target/shaders.ir
