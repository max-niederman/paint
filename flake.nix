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
            cargo-flamegraph
            cargo-watch
            cargo-edit

            # Tokio
            protobuf

            # Mold
            mold

            # Varnish
            wasm-pack
            (rustPlatform.buildRustPackage rec {
              # pasted until NixOS/nixpkgs#152595 is merged
              pname = "perseus-cli";
              version = "0.3.0";

              src = fetchCrate {
                inherit pname version;
                sha256 = "sha256-YyQQjuxNUxuo2PFluGyT/CpG22tgjRCfmFKA5MFRgHo=";
              };

              cargoSha256 = "sha256-SKxPsltXFH+ENexn/KDD43hGLSTgvtU9hv9Vdi2oeFA=";

              meta = with lib; {
                homepage = "https://arctic-hen7.github.io/perseus";
                description = "A high-level web development framework for Rust with full support for server-side rendering and static generation";
                maintainers = with maintainers; [ max-niederman ];
                license = with licenses; [ mit ];
              };
            })
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
