{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    crane = {
      url = "github:ipetkov/crane";
    };
    crane.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
    };
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ self, crane, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {

      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];
      systems = [
        "x86_64-linux"
      ];

      perSystem = { config, self', inputs', pkgs, system, ... }:
        let
          manifest = (pkgs.lib.importTOML ./Cargo.toml).package;

          craneLib = crane.mkLib pkgs;
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          commonArgs = {
            inherit src;
          };

          # The libraries.
          # This is meant to be reused
          # so as to not recompile it everytime.
          cargoArtifacts = craneLib.buildDepsOnly
            (
              commonArgs //
              {
                pname = manifest.name;
              }
            );

          # The flavours derivation.
          # This is the package that gets consumed by nix.
          flavoursDrv = craneLib.buildPackage
            (
              commonArgs //
              {
                inherit cargoArtifacts;
              }
            );
        in
        rec
        {
          # The checks for the package.
          # This should help in development.
          checks = {
            inherit flavoursDrv;

            # This runs clippy on the source code.
            flavours-clippy = craneLib.cargoClippy
              (
                commonArgs //
                {
                  inherit cargoArtifacts;
                  cargoClippyExtraArgs = "--all-targets -- --deny warnings";
                }
              );
          };

          _module.args = {
            pkgs = import inputs.nixpkgs {
              # This makes the nixpkgs be the one
              # suited to your current development system;
              inherit system;

              overlays = with inputs; [
                rust-overlay.overlays.default
              ];
            };
          };
          # This generates a overlay
          # You should consume this in your flake if you want flavours.
          overlayAttrs = {
            inherit (config.packages) flavours;
          };

          packages = {
            flavours = flavoursDrv;
          };
          packages.default = packages.flavours;

          devShells = {
            development = pkgs.mkShell {
              buildInputs = with pkgs; [
                rust-bin.stable.latest.default
              ];
            };
          };
          devShells.default = devShells.development;

        };

    };
}

