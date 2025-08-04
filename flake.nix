{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rustToolchain = fenix.packages.${system}.stable.toolchain;
        rustPackage = fenix.packages.${system}.stable.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rust-analyzer"
        ];
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        build = import ./build.nix { inherit pkgs rustPlatform; };
      in
      {
        packages = {
          sensors = build.main-package;
          docker-image = build.docker-image;
          default = build.main-package;
        };

        devShell = pkgs.mkShell {
          buildInputs = [
            rustPackage
            fenix.packages.${system}.latest.rustfmt
            pkgs.nixfmt-rfc-style
          ];
        };
      }
    );
}
