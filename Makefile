.PHONY: dist

dist:
	cd kiro && make dist

version:
	test -n "$(VERSION)"
	sed -i 's/^  version.*/  version = "$(VERSION)"/g' ./kiro/Cargo.toml
	sed -i 's/^  version.*/  version = "$(VERSION)"/g' ./api/rust/Cargo.toml

	cd api && make
	make test
	git add .
	git commit -v -m "Bump version to $(VERSION)"
	git tag -a v$(VERSION) -m "v$(VERSION)"

api: version
		cd api && make

devshell:
	nix-shell

docker-devshell:
	docker compose run --rm --service-ports --name kiro kiro

test:
	cd api && make rust
	cd kiro && make test

test-all:
	cd api && make rust
	cd kiro && make test-all
