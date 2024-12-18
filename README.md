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
# Windows (PowerShell)
.\make.ps1                  # Run the server
.\make.ps1 dev              # Start dev environment
.\make.ps1 version x.x.x    # Update version
.\make.ps1 docker-dev       # Start Docker development environment
.\make.ps1 test             # Run tests
.\make.ps1 test-all         # Run all tests including integration
.\make.ps1 help             # Show available commands
.\make.ps1 config           # Configure the server

# macOS and Linux
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
# Windows (PowerShell)
.\make.ps1

# macOS and Linux
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
# Windows (PowerShell)
.\make.ps1 dev

# macOS and Linux
make dev
```

To use the specific Nixpkgs version (nixos-24.05), you can run:

```bash
nix-shell -I nixpkgs=https://github.com/NixOS/nixpkgs/archive/nixos-24.05.tar.gz
```

### 🧹 Nix Maintenance

To clean the Nix store:

```bash
nix-store --gc
```

To add user permissions to the Nix folder:

```bash
# Windows (PowerShell)
icacls C:\nix /grant "$env:USERNAME:(OI)(CI)F" /T

# macOS and Linux
sudo chown --recursive "$USER" /nix
```

### 👥 Nix Multi-User Setup

To set up Nix with multi-user support:

1. Install Nix in multi-user mode:
   ```bash
   # Windows (PowerShell)
   iex ((New-Object System.Net.WebClient).DownloadString('https://nixos.org/nix/install')) -daemon

   # macOS and Linux
   sh <(curl -L https://nixos.org/nix/install) --daemon
   ```

2. Create and start the nix-daemon service:
   ```bash
   # Windows (PowerShell)
   # Not applicable, Windows uses a different service management system

   # macOS and Linux
   sudo mkdir -p /etc/systemd/system
   sudo curl -o /etc/systemd/system/nix-daemon.service https://raw.githubusercontent.com/NixOS/nix/master/etc/systemd/nix-daemon.service
   sudo systemctl enable nix-daemon
   sudo systemctl start nix-daemon
   ```

3. Add Nix bin to PATH:
   ```bash
   # Windows (PowerShell)
   [Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\nix\usr\bin", [EnvironmentVariableTarget]::Machine)

   # macOS and Linux
   echo 'export PATH=$PATH:/nix/var/nix/profiles/default/bin' | sudo tee -a /etc/profile
   ```

4. Create nixbld group and add users:
   ```bash
   # Windows (PowerShell)
   # Not applicable, Windows uses a different user/group management system

   # macOS
   sudo dscl . -create /Groups/nixbld
   sudo dscl . -create /Groups/nixbld PrimaryGroupID 301
   sudo dscl . -create /Users/nixbld1 UniqueID 301
   sudo dscl . -create /Users/nixbld1 PrimaryGroupID 301
   sudo dscl . -append /Groups/nixbld GroupMembership nixbld1
   sudo dscl . -append /Groups/nixbld GroupMembership yourusername

   # Linux
   sudo groupadd nixbld
   sudo usermod -a -G nixbld yourusername
   ```

5. Set up Nix configuration:
   ```bash
   # Windows (PowerShell)
   New-Item -ItemType Directory -Force -Path "$env:APPDATA\nix"
   Add-Content "$env:APPDATA\nix\nix.conf" "build-users-group = nixbld"

   # macOS and Linux
   sudo mkdir -p /etc/nix
   echo "build-users-group = nixbld" | sudo tee -a /etc/nix/nix.conf
   ```

6. Restart your shell or log out and back in.

For MacOS users, some commands might differ. Always refer to the official Nix documentation for the most up-to-date instructions.

Happy coding! 🎈🎊
