# 🚀 API-Client

API-Client is a powerful and flexible gRPC server designed to seamlessly interact with the [Service Proto](https://github.com/Linzell/SRC-Proto). This project provides a robust interface for communication between clients and the Service Proto, enabling efficient and streamlined data exchange.

[![CI](https://github.com/Linzell/API-Client/actions/workflows/CI.yml/badge.svg?branch=main)](https://github.com/Linzell/API-Client/actions/workflows/CI.yml)
[![codecov](https://codecov.io/gh/Linzell/API-Client/branch/main/graph/badge.svg?token=4TBIXUE2YV)](https://codecov.io/gh/Linzell/API-Client)

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![gRPC](https://img.shields.io/badge/gRPC-244c5a?style=for-the-badge&logo=grpc&logoColor=white)](https://grpc.io/)
[![Tonic](https://img.shields.io/badge/Tonic-00ADD8?style=for-the-badge&logo=rust&logoColor=white)](https://github.com/hyperium/tonic)

## 🛠️ Installation

### 📦 Requirements

- 🐳 [Docker](https://www.docker.com/)
- ❄️ [Nix](https://nixos.org/download.html) (optional)

### 🔑 Set Up Environment Variables

Create a `.env` file in the project root with these variables:

```bash
# Server Configuration
PORT=3000
FRONT_CONNECT_URL=http://localhost:5173
CERT_PEM_URL="certs/cert.pem"
KEY_PEM_URL="certs/key.pem"
JAEGER_AGENT_HOST="http://localhost:4317"

# Add any other necessary environment variables
```

## 🚀 Getting Started

### 🛠️ Using the Makefile

Useful commands:

```bash
make                        # Run the server
make dev                    # Start dev environment
make version x.x.x          # Update version
make docker-dev             # Start Docker development environment
make test                   # Run tests
make test-all               # Run all tests including integration
make help                   # Show available commands
make config                 # Configure the server
```

### 🐳 Docker Setup

```bash
docker compose up -d
```

Build Docker image:
```bash
docker compose build
```

### 🖥️ Manual Setup

Run the server:
```bash
make
```

### 📊 Run OpenTelemetry

```bash
docker compose up -d jaeger
```

Access [Jaeger UI](http://localhost:16686/)

### 🛠️ Nix Setup

If you have Nix installed, you can use it to set up the development environment:

```bash
make dev
```

### 🧹 Nix Maintenance

To clean the Nix store:

```bash
nix-store --gc
```

To add user permissions to the Nix folder:

```bash
sudo chown --recursive "$USER" /nix
```

### 👥 Nix Multi-User Setup

To set up Nix with multi-user support:

1. Install Nix in multi-user mode:
   ```bash
   sh <(curl -L https://nixos.org/nix/install) --daemon
   ```

2. Create and start the nix-daemon service:
   ```bash
   sudo mkdir -p /etc/systemd/system
   sudo curl -o /etc/systemd/system/nix-daemon.service https://raw.githubusercontent.com/NixOS/nix/master/etc/systemd/nix-daemon.service
   sudo systemctl enable nix-daemon
   sudo systemctl start nix-daemon
   ```

3. Add Nix bin to PATH:
   ```bash
   echo 'export PATH=$PATH:/nix/var/nix/profiles/default/bin' | sudo tee -a /etc/profile
   ```

4. Create nixbld group and add users:
   ```bash
   sudo groupadd nixbld
   sudo usermod -a -G nixbld yourusername
   ```

5. Set up Nix configuration:
   ```bash
   sudo mkdir -p /etc/nix
   echo "build-users-group = nixbld" | sudo tee -a /etc/nix/nix.conf
   ```

6. Restart your shell or log out and back in.

For MacOS users, some commands might differ. Always refer to the official Nix documentation for the most up-to-date instructions.

Happy coding! 🎈🎊
