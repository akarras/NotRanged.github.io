wasm:
	wasm-pack build -t no-modules --release --no-typescript . -Z build-std=panic_abort,std
	rm -rf ../app/lib/pkg/
	cp -a pkg/ ../app/lib/pkg/
wasm-thicc:
	RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' wasm-pack build --target no-modules --features wasm-thread -Z build-std=panic_abort,std
	rm -rf ../app/lib/pkg
	cp -a pkg/ ../app/lib/pkg/
wasm-web:
	RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' wasm-pack build --target web --features wasm-thread --features wasm-genevo -Z build-std=panic_abort,std
	rm -rf ../app/lib/pkg
	cp -a pkg/ ../app/lib/pkg/
