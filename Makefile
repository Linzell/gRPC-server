.PHONY: dist api

dist: docker-services
	cd kiro && make dist

help: docker-services
	cd kiro && make help

config: docker-services
	cd kiro && make config

version:
	test -n "$(VERSION)"
	sed -i 's/^  version.*/  version = "$(VERSION)"/g' ./kiro/Cargo.toml
	sed -i 's/^  version.*/  version = "$(VERSION)"/g' ./api/rust/Cargo.toml

	cd api && make
	make test
	git add .
	git commit -v -m "Bump version to $(VERSION)"
	git tag -a v$(VERSION) -m "v$(VERSION)"

docker-services:
	@docker ps -a | grep -q surrealdb || (echo "Starting SurrealDB..." && docker-compose up -d surrealdb)
	@docker ps -a | grep -q jaeger || (echo "Starting Jaeger..." && docker-compose up -d jaeger)

api: version
		cd api && make

dev:
	git submodule update --init --recursive
	nix-shell

docker-dev:
	docker compose run --rm --service-ports --name kiro kiro

test:
	cd api && make rust
	cd kiro && make test

test-all:
	cd api && make rust
	cd kiro && make test-all
