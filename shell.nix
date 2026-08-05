{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/nixos-26.05.tar.gz") {} }:

pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.pkg-config
  ];
  buildInputs = [
    pkgs.rustup
    pkgs.cargo-llvm-cov
    pkgs.sqlite-interactive
    pkgs.sqlx-cli
    pkgs.just
    pkgs.jq

    pkgs.mdbook
    pkgs.mdbook-mermaid
    pkgs.dprint

    pkgs.nodejs
    pkgs.pnpm
  ];
  shellHook = ''
    export PATH="$PWD/.cargo/bin:$PATH"
    export DATABASE_URL="sqlite://$PWD/nanopulse-server/nanopulse_test.sqlite"
  '';
  DOCKER_BUILDKIT = "1";
  NIX_STORE = "/nix/store";
}

