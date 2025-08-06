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
        rustToolchain = fenix.packages.${system}.stable.minimalToolchain;
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
      in
      {
        packages.ndeploy = rustPlatform.buildRustPackage {
          pname = "ndeploy";
          version = "0.1.0";

          buildInputs = [ pkgs.nix-output-monitor ];

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./src
            ];
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          NOM_PATH = "${pkgs.nix-output-monitor}/bin/nom";
        };

        packages.default = self.packages."${system}".ndeploy;

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
