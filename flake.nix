{
  description = "Build a cargo workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;

        # Use workspace root directly, since it is a proper cargo workspace
        src = ./.;

        commonArgs = {
          src = ./.;
          strictDeps = true;

          buildInputs =
            [
              # Add additional build inputs here
            ] ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          pname = "my-workspace";
          version = "0.0.0";
        });

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          doCheck = false;
        };

        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              (craneLib.fileset.commonCargoSources ./rpg-combat-tui)
              (craneLib.fileset.commonCargoSources ./player-client)
              (craneLib.fileset.commonCargoSources ./core)
              # (craneLib.fileset.commonCargoSources crate)
            ];
          };

        rpg-combat-tui = craneLib.buildPackage (
          individualCrateArgs // {
            pname = "rpg-combat-tui";
            version = "0.1.0";
            cargoExtraArgs = "-p rpg-combat-tui";
            src = fileSetForCrate ./rpg-combat-tui;
          }
        );

        player-client = craneLib.buildPackage (
          individualCrateArgs // {
            pname = "player-client";
            version = "0.1.0";
            cargoExtraArgs = "-p player-client";
            src = fileSetForCrate ./player-client;
          }
        );

        core = craneLib.buildPackage (
          individualCrateArgs // {
            pname = "core";
            version = "0.1.0";
            cargoExtraArgs = "-p core";
            src = fileSetForCrate ./core;
          }
        );
      in
      {
        checks = {
          # Build the crates as part of `nix flake check` for convenience
          inherit rpg-combat-tui player-client core;

          # Run clippy (and deny all warnings) on the workspace source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          my-workspace-clippy = craneLib.cargoClippy (
            commonArgs // {
              pname = "my-workspace";
              version = "0.0.0";
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          my-workspace-doc = craneLib.cargoDoc (
            commonArgs // {
              pname = "my-workspace";
              version = "0.0.0";
              inherit cargoArtifacts;
            }
          );

          # Check formatting
          my-workspace-fmt = craneLib.cargoFmt {
            inherit src;
            pname = "my-workspace-fmt";
            version = "0.0.0";
          };

          my-workspace-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            # taplo arguments can be further customized below as needed
            # taploExtraArgs = "--config ./taplo.toml";
            pname = "my-workspace-toml-fmt";
            version = "0.0.0";
          };

          # Audit dependencies
          my-workspace-audit = craneLib.cargoAudit {
            inherit src advisory-db;
            pname = "my-workspace";
            version = "0.0.0";
          };

          # Audit licenses
          my-workspace-deny = craneLib.cargoDeny {
            inherit src;
            pname = "my-workspace";
            version = "0.0.0";
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on other crate derivations
          # if you do not want the tests to run twice
          my-workspace-nextest = craneLib.cargoNextest (
            commonArgs // {
              pname = "my-workspace";
              version = "0.0.0";
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );

          # Ensure that cargo-hakari is up to date
          my-workspace-hakari = craneLib.mkCargoDerivation {
            inherit src;
            pname = "my-workspace-hakari";
            version = "0.0.0";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo hakari generate --diff
              cargo hakari manage-deps --dry-run
              cargo hakari verify
            '';

            nativeBuildInputs = [
              pkgs.cargo-hakari
            ];
          };
        };

        packages = {
          inherit rpg-combat-tui player-client core;
        };

        apps = {
          rpg-combat-tui = flake-utils.lib.mkApp {
            drv = rpg-combat-tui;
          };
          player-client = flake-utils.lib.mkApp {
            drv = player-client;
          };
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [ pkgs.cargo-hakari pkgs.rust-analyzer ];
        };
      }
    );
}

