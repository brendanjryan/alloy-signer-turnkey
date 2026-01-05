.PHONY: check test build fix clean doc

check:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-features
	cargo build --no-default-features

test:
	cargo test --all-features --verbose

build:
	cargo build --release

fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

clean:
	cargo clean

doc:
	cargo doc --all-features --open
