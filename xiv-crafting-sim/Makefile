wasm:
	wasm-pack build -t web --release --no-typescript . -Z build-std=panic_abort,std
	rm -rf ../app/lib/xiv-craft-simulator
	rm ./pkg/.gitignore
	cp -a pkg/ ../app/lib/xiv-craft-simulator
wasm-thread:
	RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' wasm-pack build --release --target web --features wasm-thread --features wasm-genevo -Z build-std=panic_abort,std
	rm -rf ../app/lib/xiv-thread-simulator
	rm ./pkg/.gitignore
	cp -a pkg/ ../app/lib/xiv-thread-simulator/
wasm-simd:
	RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128' wasm-pack build --release --target web --features wasm-thread --features wasm-genevo -Z build-std=panic_abort,std
	rm -rf ../app/lib/xiv-thread-simd
	rm ./pkg/.gitignore
	cp -a pkg/ ../app/lib/xiv-thread-simd/
all: wasm wasm-thread
