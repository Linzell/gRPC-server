.PHONY: dist

PKG_VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

dist:
	    cross build --target aarch64-unknown-linux-musl --release
	    cross build --target x86_64-unknown-linux-musl --release
	    cross build --target armv7-unknown-linux-musleabihf --release

		cargo deb --target x86_64-unknown-linux-musl --no-build --no-strip
		cargo deb --target armv7-unknown-linux-musleabihf --no-build --no-strip
		cargo deb --target aarch64-unknown-linux-musl --no-build --no-strip

		cargo generate-rpm --target x86_64-unknown-linux-musl --target-dir ../target
		cargo generate-rpm --target armv7-unknown-linux-musleabihf --target-dir ../target
		cargo generate-rpm --target aarch64-unknown-linux-musl --target-dir ../target

		mkdir -p ../dist

		cp ../target/x86_64-unknown-linux-musl/debian/*.deb ../dist
		cp ../target/armv7-unknown-linux-musleabihf/debian/*.deb ../dist
		cp ../target/aarch64-unknown-linux-musl/debian/*.deb ../dist

		cp ../target/x86_64-unknown-linux-musl/generate-rpm/*.rpm ../dist
		cp ../target/armv7-unknown-linux-musleabihf/generate-rpm/*.rpm ../dist
		cp ../target/aarch64-unknown-linux-musl/generate-rpm/*.rpm ../dist

		tar -czvf ../dist/kiro_$(PKG_VERSION)_amd64.tar.gz -C ../target/x86_64-unknown-linux-musl/release kiro
		tar -czvf ../dist/kiro_$(PKG_VERSION)_armv7hf.tar.gz -C ../target/armv7-unknown-linux-musleabihf/release kiro
		tar -czvf ../dist/kiro_$(PKG_VERSION)_arm64.tar.gz -C ../target/aarch64-unknown-linux-musl/release kiro

test:
	    cargo fmt --check
		cargo clippy --no-deps
		TZ=UTC cargo test

test-all:
	    cargo fmt --check
		cargo clippy --no-deps
		TZ=UTC cargo test --features test-all-integrations
