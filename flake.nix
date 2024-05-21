{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = inputs @ {
    self,
    crane,
    flake-parts,
    advisory-db,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.flake-parts.flakeModules.easyOverlay
      ];
      systems = [
        "x86_64-linux"
        "i686-linux"
        "aarch64-linux"
      ];

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource (craneLib.path ./.);

        commonArgs = {
          inherit src;
          pname = "flavours";
          version = "0.7.1";
        };

        cargoArtifacts =
          craneLib.buildDepsOnly
          commonArgs;

        flavoursDrv =
          craneLib.buildPackage
          (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = "flavours";
            }
          );
      in rec
      {
        checks = {
          docs = craneLib.cargoDoc (commonArgs
            // {
              inherit cargoArtifacts;
            });

          fmt = craneLib.cargoFmt (commonArgs
            // {
              inherit src;
            });

          audit = craneLib.cargoAudit (commonArgs
            // {
              inherit src advisory-db;
            });

          deny = craneLib.cargoDeny (commonArgs
            // {
              inherit src;
            });

          clippy =
            craneLib.cargoClippy
            (
              commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              }
            );

          nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );
        };

        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
          ];
        };

        overlayAttrs = {
          inherit (config.packages) flavours;
        };

        devShells = {
          development = craneLib.devShell {
            checks = self.checks;
            buildInputs = with pkgs; [
              bacon
              cargo-deny
              cargo-edit
              cargo-lock
              rust-analyzer
              cargo-nextest
            ];
          };
        };
        devShells.default = devShells.development;

        packages = {
          flavours = flavoursDrv;
        };
        packages.default = packages.flavours;

        formatter = pkgs.writeShellApplication {
          name = "treefmt";
          runtimeInputs = with pkgs; [
            treefmt
            rustfmt
            alejandra
          ];
          text = ''
            exec treefmt "$@"
          '';
        };
      };
    };
}
