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
        rust = with fenix.packages.${system}; rec {
          wasm = targets.wasm32-unknown-unknown.latest;
          native = latest;
          dev.toolchain = combine [ wasm.toolchain native.toolchain rust-analyzer ];
        };
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            # Rust
            rust.dev.toolchain
            mold
            wasm-pack

            # JS
            nodePackages.pnpm
          ];

          # needed for rust-openssl
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          LD_LIBRARY_PATH = nixpkgs.lib.strings.makeLibraryPath (with pkgs; [ openssl ]);

          # redirect ld calls to mold
          MOLD_PATH = "${pkgs.mold}/bin/mold";
          LD_PRELOAD = "${pkgs.mold}/lib/mold/mold-wrapper.so";
        };
      }
    );
}
