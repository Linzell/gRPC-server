.PHONY:

test:
	cargo fmt --check
	cargo clippy --no-deps
	TZ=UTC cargo test
