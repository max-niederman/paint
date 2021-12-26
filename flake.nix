{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.url = "nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rust = with fenix.packages.${system}; complete;
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            # Rust
            rust.toolchain
            rust-analyzer
            cargo-flamegraph
            cargo-watch
            cargo-edit

            # Tokio
            protobuf

            # Mold
            mold

            # JavaScript
            nodejs-16_x
            nodePackages.pnpm
          ];

          # needed for rust-openssl
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          LD_LIBRARY_PATH = nixpkgs.lib.strings.makeLibraryPath [ pkgs.openssl ];

          # redirect ld calls to mold
          MOLD_PATH = "${pkgs.mold}/bin/mold";
          LD_PRELOAD = "${pkgs.mold}/lib/mold/mold-wrapper.so";
        };
      }
    );
}
