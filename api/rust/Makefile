.PHONY: proto version test

all: proto test

proto:
	rm -rf proto
	mkdir -p proto/kiro
	mkdir -p proto/google
	cp -r ../SRC-Proto/proto/* proto/kiro/
	mv proto/kiro/google proto/google

test:
	cargo test

publish:
	cargo publish
