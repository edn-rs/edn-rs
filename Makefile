integration:
	cargo test --test lib --no-fail-fast --features "json"

unit:
	cargo test --locked  --no-fail-fast --lib

.PHONY: examples
examples:
	cargo test --examples --no-fail-fast
	cargo test --example json_to_edn --features "json"
	cargo run --example async --features "async"

test: unit integration examples