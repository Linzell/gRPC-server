# API-Client

This is a gRPC server that can be used to interact with the [Service Proto](https://github.com/Linzell/SRC-Proto).

[![CI](https://github.com/Linzell/API-Client/actions/workflows/CI.yml/badge.svg?branch=main)](https://github.com/Linzell/API-Client/actions/workflows/CI.yml)

## Running the server locally

### Requirements

Running the server requires the following:

- [Docker](https://www.docker.com/)
- [Nix](https://nixos.org/download.html) (optional)

### Running environment with Nix

Nix is a package manager that can be used to install the required dependencies for the project. To run the server with Nix, run the following commands:

```bash
make dev
```

### Running environment without Nix

If you do not have Nix installed, you can run the server with Docker. To run the server with Docker, run the following commands:

```bash
make docker-dev
```

### Running the server

After setting up the environment, you can run the server.

To run the server, run the following command:

```bash
make
```

## CLI

The server has a CLI that can be used to interact with the server. To see the available commands, run the following command:

```bash
make help
```

### Configuring the server with the CLI

The server can be configured with the CLI. To configure the server, run the following command:

```bash
make config
```

## Running the tests

### Test features without integration

To run the tests, run the following command:

```bash
make test
```

### Test all features

To run the tests with integration, run the following command:

```bash
make test-all
```

## Creating a version

To create a version, run the following command:

```bash
make version <VERSION>
```

The `VERSION` is the version number that you want to create ( e.g. 0.1.0).

## Nix commands

### Clean store

To clean the Nix store, run the following command:

```bash
nix-store --gc
```
