.PHONY: proto test

proto:
	rm -rf proto
	mkdir -p proto
	cp -r ../SRC-Proto/proto/**/ proto/

test:
	cargo fmt --check
	cargo clippy --no-deps
	TZ=UTC cargo test

test-all:
	cargo fmt --check
	cargo clippy --no-deps
	TZ=UTC cargo test --all-features -- --nocapture
