.PHONY: dist

dist:
	    cargo run --release -- --config config.toml

help:
	    cargo run --release -- --config config.toml help

config:
	    cargo run --release -- --config config.toml configfile

test:
	    cargo fmt --check
		cargo clippy --no-deps
		TZ=UTC cargo test

test-all:
		cargo fmt --check
		cargo clippy --no-deps
		TZ=UTC cargo test --features test-all-integrations
