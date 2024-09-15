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

### Add user permissions to the Nix folder

To add user permissions to the Nix store, run the following command:

```bash
sudo chown --recursive "$USER" /nix
```

### Add Nix multi-user support

To set up Nix with multi-user support, follow these steps:

1. Install Nix in multi-user mode:

   ```bash
   sh <(curl -L https://nixos.org/nix/install) --daemon
   ```

2. Create the nix-daemon service:

   ```bash
   sudo mkdir -p /etc/systemd/system
   sudo curl -o /etc/systemd/system/nix-daemon.service https://raw.githubusercontent.com/NixOS/nix/master/etc/systemd/nix-daemon.service
   ```

3. Enable and start the nix-daemon service:

   ```bash
   sudo systemctl enable nix-daemon
   sudo systemctl start nix-daemon
   ```

4. Add the Nix bin directory to the system PATH:

   ```bash
   echo 'export PATH=$PATH:/nix/var/nix/profiles/default/bin' | sudo tee -a /etc/profile
   ```

5. Create a group for Nix users:

   ```bash
   sudo groupadd nixbld
   ```

6. Add users to the nixbld group:

   ```bash
   sudo usermod -a -G nixbld yourusername
   ```

7. Set up the Nix environment for all users:

   ```bash
   sudo mkdir -p /etc/nix
   echo "build-users-group = nixbld" | sudo tee -a /etc/nix/nix.conf
   ```

8. Restart your shell or log out and log back in for changes to take effect.

After completing these steps, Nix should be set up for multi-user support. Multiple users on the system can now use Nix independently, with a shared Nix store.

Remember to replace 'yourusername' with the actual username you want to add to the nixbld group. You may need to repeat step 6 for each user who should have access to Nix.

Note: The exact steps might vary slightly depending on your operating system and configuration. Always refer to the official Nix documentation for the most up-to-date instructions.
