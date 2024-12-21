.PHONY: dist api

dist: docker-services
	cd kiro && $(MAKE) dist

docker-services:
	@if [ "$(OS)" = "Windows_NT" ]; then \
		powershell -Command "if (-not (docker ps -a | Select-String surrealdb)) { echo 'Starting SurrealDB...'; docker-compose up -d surrealdb }"; \
		powershell -Command "if (-not (docker ps -a | Select-String jaeger)) { echo 'Starting Jaeger...'; docker-compose up -d jaeger }"; \
	else \
		docker ps | grep -q surrealdb || (echo "Starting SurrealDB..." && docker-compose up -d surrealdb); \
		docker ps | grep -q jaeger || (echo "Starting Jaeger..." && docker-compose up -d jaeger); \
	fi

api:
	cd kiro-api && $(MAKE)

dev:
	git submodule update --init --recursive
	$(if $(filter $(OS),Windows_NT), \
		(echo "Nix is not natively supported on Windows. Please use WSL or Docker." && \
		powershell -Command "icacls C:\nix /grant \"$env:USERNAME:(OI)(CI)F\" /T"), \
		$(if $(filter $(shell uname),Linux), \
			(sudo chown --recursive "$$USER" /nix && nix-shell), \
			(nix-shell) \
		) \
	)

test:
	@echo "Running quality checks across all crates..."
	@echo "Running tests..."
	cd kiro-api && $(MAKE) test
	cd kiro && $(MAKE) test
	cd kiro-auth && $(MAKE) test
	cd kiro-client && $(MAKE) test
	cd kiro-database && $(MAKE) test
	@echo "Running security audit..."
	cargo audit
	@echo "All quality checks completed."

test-all:
	@echo "Running quality checks across all crates..."
	@echo "Running tests..."
	cd kiro-api && $(MAKE) test-all
	cd kiro && $(MAKE) test-all
	cd kiro-auth && $(MAKE) test-all
	cd kiro-client && $(MAKE) test-all
	cd kiro-database && $(MAKE) test-all
	@echo "Running security audit..."
	cargo audit
	@echo "All quality checks completed."
