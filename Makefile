.PHONY: dist api

dist:
	cd kiro && $(MAKE) dist

help: docker-services
	cd kiro && $(MAKE) help

config: docker-services
	cd kiro && $(MAKE) config

version:
	@if [ -z "$(filter-out $@,$(MAKECMDGOALS))" ]; then \
		echo "Error: VERSION is not set. Please provide a version number."; \
		exit 1; \
	fi
	@$(eval VERSION := $(filter-out $@,$(MAKECMDGOALS)))
	$(if $(filter $(OS),Windows_NT), \
		(powershell -Command "(Get-Content ./kiro/Cargo.toml) -replace '^version = .*', 'version = \"$(VERSION)\"' | Set-Content ./kiro/Cargo.toml" && \
		powershell -Command "(Get-Content ./api/rust/Cargo.toml) -replace '^version = .*', 'version = \"$(VERSION)\"' | Set-Content ./api/rust/Cargo.toml"), \
		$(if $(filter $(shell uname),Darwin), \
			(sed -i '' 's/^version = .*/version = "$(VERSION)"/g' ./kiro/Cargo.toml && \
			sed -i '' 's/^version = .*/version = "$(VERSION)"/g' ./api/rust/Cargo.toml), \
			(sed -i 's/^version = .*/version = "$(VERSION)"/g' ./kiro/Cargo.toml && \
			sed -i 's/^version = .*/version = "$(VERSION)"/g' ./api/rust/Cargo.toml) \
		) \
	)

	cd api && $(MAKE)
	$(MAKE) test
	git add .
	git commit -v -m "Bump version to $(VERSION)"
	git tag -a v$(VERSION) -m "v$(VERSION)"

docker-services:
	@if [ "$(OS)" = "Windows_NT" ]; then \
		powershell -Command "if (-not (docker ps -a | Select-String surrealdb)) { echo 'Starting SurrealDB...'; docker-compose up -d surrealdb }"; \
		powershell -Command "if (-not (docker ps -a | Select-String jaeger)) { echo 'Starting Jaeger...'; docker-compose up -d jaeger }"; \
	else \
		docker ps | grep -q surrealdb || (echo "Starting SurrealDB..." && docker-compose up -d surrealdb); \
		docker ps | grep -q jaeger || (echo "Starting Jaeger..." && docker-compose up -d jaeger); \
	fi

api: version
		cd api && $(MAKE)

dev: docker-services
	git submodule update --init --recursive
	$(if $(filter $(OS),Windows_NT), \
		(echo "Nix is not natively supported on Windows. Please use WSL or Docker." && \
		powershell -Command "icacls C:\nix /grant \"$env:USERNAME:(OI)(CI)F\" /T"), \
		(nix-shell && sudo chown --recursive "$$USER" /nix) \
	)

docker-dev:
	docker compose run --rm --service-ports --name kiro kiro

test:
	cd api && $(MAKE) rust
	cd kiro && $(MAKE) test

test-all:
	cd api && $(MAKE) rust
	cd kiro && $(MAKE) test-all
