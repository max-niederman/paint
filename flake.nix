{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.url = "nixpkgs/nixos-21.11";

    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, naersk, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          inherit (pkgs) lib;

          rust = pkgs.rust-bin.selectLatestNightlyWith
            (toolchain: toolchain.default.override {
              extensions = [ "rust-src" ];
            });
          naersk-lib = naersk.lib.${system}.override
            (lib.attrsets.genAttrs [ "cargo" "rustc" ] (_: rust));
        in
        rec {
          packages = rec {
            paint = naersk-lib.buildPackage {
              pname = "paint";
              root = ./.;
              copyBins = true;
              copyLibs = true;
            };
          };
          defaultPackage = packages.paint;

          devShell = pkgs.mkShell {
            buildInputs = with pkgs;
              [
                docker-compose

                # Rust
                rust
                rust-analyzer
                rustfmt
                mold
                bintools

                # JS
                nodejs-16_x
                nodePackages.pnpm
                nodePackages.prettier
              ];

            # redirect ld calls to mold
            MOLD_PATH = "${pkgs.mold}/bin/mold";
            LD_PRELOAD = "${pkgs.mold}/lib/mold/mold-wrapper.so";

            # set log level for Rust crates
            RUST_LOG = builtins.concatStringsSep
              ","
              [
                "info"
                "canvas_lms=debug"
                "oil=debug"
              ];
          };
        }
      );
}
