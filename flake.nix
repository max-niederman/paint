{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.url = "nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rust = with fenix.packages.${system}; default;
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            # Rust
            rust.toolchain
            rust-analyzer

            # JavaScript
            nodejs-16_x
            nodePackages.pnpm
          ];
        };
      }
    );
}
