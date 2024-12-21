.PHONY:

test:
	cargo fmt --check
	cargo clippy --no-deps
	TZ=UTC cargo test

test-all:
	cargo fmt --check
	cargo clippy --no-deps
	TZ=UTC cargo test --all-features -- --nocapture
