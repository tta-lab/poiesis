.PHONY: build test check fmt clippy install reinstall release clean

build:
	cargo build --all

release:
	cargo build --release --all

test:
	cargo test --all

check: fmt clippy test

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets -- -D warnings

install:
	cargo install --path poiesis-cli

reinstall:
	cargo install --path poiesis-cli --force

clean:
	cargo clean
