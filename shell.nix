{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-24.05.tar.gz") {}, lib ? pkgs.lib }:

pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = [
    pkgs.cacert
    pkgs.rustup
    pkgs.protobuf
    pkgs.postgresql

    # Code quality tools
    pkgs.cargo-audit
    pkgs.cargo-watch
    pkgs.cargo-tarpaulin
    pkgs.cargo-cache
    pkgs.clippy
    pkgs.rustfmt

    # Development tools
    pkgs.starship
    pkgs.zsh
    pkgs.oh-my-zsh
    pkgs.fzf
    pkgs.ripgrep
    pkgs.eza
    pkgs.bat
    pkgs.fd
    pkgs.htop
    pkgs.tmux
    pkgs.direnv
    pkgs.bash-completion
  ]
  # Required for mac
  ++ lib.optionals pkgs.stdenv.isDarwin [
    # Additional darwin specific inputs can be set here
    pkgs.libiconv
    pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
  ];

  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
  BINDGEN_EXTRA_CLANG_ARGS = "-I${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.llvmPackages.libclang.version}/include";
  DOCKER_BUILDKIT = "1";
  NIX_STORE = "/nix/store";

  shellHook = ''
    export PATH=$PATH:~/.local/bin
    export CARGO_HOME=$HOME/.cargo
    export PATH=$CARGO_HOME/bin:$PATH
    unset SOURCE_DATE_EPOCH

    # Install required Rust components
    rustup component add rustfmt clippy rust-src

    eval "$(starship init bash)"

    source ${pkgs.fzf}/share/fzf/completion.bash
    source ${pkgs.fzf}/share/fzf/key-bindings.bash

    source ${pkgs.bash-completion}/etc/profile.d/bash_completion.sh

    source ${pkgs.nix}/etc/profile.d/nix.sh

    eval "$(direnv hook bash)"

    complete -C ${pkgs.starship}/bin/starship starship

    alias ls='eza --color=auto'
    alias ll='eza -l'
    alias cat='bat'
    alias find='fd'

    echo "✨ Welcome to your enhanced Rust development environment! ✨"
    echo "Development tools installed:"
    echo "- Code Quality: cargo-audit, cargo-watch, cargo-tarpaulin, clippy, rustfmt"
    echo "- Utils: starship, fzf, ripgrep, eza, bat, fd"
  '';
}
