{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
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
        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      {
        packages.ndeploy = rustPlatform.buildRustPackage {
          pname = manifest.name;
          version = manifest.version;

          buildInputs = [ pkgs.nix-output-monitor ];
          nativeBuildInputs = [ pkgs.installShellFiles ];

          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./src
              ./build.rs
            ];
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          NOM_PATH = "${pkgs.nix-output-monitor}/bin/nom";
          NIXOS_REBUILD_PATH = "${pkgs.nixos-rebuild}/bin/nixos-rebuild";

          postInstall = ''
            ls target/completions/*
            installShellCompletion target/completions/ndeploy.bash target/completions/ndeploy.fish target/completions/_ndeploy
          '';
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
