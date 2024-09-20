{
  description = "Modular graph synthesis engine";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      fenix,
      flake-utils,
      advisory-db,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p: p.rust-bin.nightly.latest.default.override { extensions = [ "rust-src" ]; }
        );
        src = craneLib.cleanCargoSource ./.;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs =
            with pkgs;
            [
              alsa-lib
              libGL
              libjack2
              xorg.libX11
              xorg.libXcursor
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
              # Additional darwin specific inputs can be set here
              pkgs.libiconv
            ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            python3
          ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        craneLibLLvmTools = craneLib.overrideToolchain (
          fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]
        );

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          # NB: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;
        };

        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              crate
            ];
          };

        dagrid-plugin = craneLib.buildPackage (
          individualCrateArgs
          // {
            inherit cargoArtifacts;

            pname = "dagrid-plugin";
            cargoExtraArgs = "-p dagrid-plugin-export";
            src = fileSetForCrate ./.;
            installPhaseCommand = ''
              cargo run -p dagrid-xtask -- bundle dagrid-plugin-export;
              mkdir -p $out;
              mv /build/source/target/bundled/dagrid-plugin-export.clap $out/dagrid.clap
            '';
          }
        );

        dagrid-standalone = craneLib.buildPackage (
          individualCrateArgs
          // {
            inherit cargoArtifacts;

            pname = "dagrid";
            src = fileSetForCrate ./.;
          }
        );
      in
      {

        checks = {
          inherit dagrid-plugin dagrid-standalone;

          check-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          check-doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });

          check-fmt = craneLib.cargoFmt { inherit src; };

          check-audit = craneLib.cargoAudit { inherit src advisory-db; };

          check-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );
        };

        packages =
          {
            inherit dagrid-plugin;
            default = dagrid-standalone;
          }
          // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // { inherit cargoArtifacts; });
          };

        apps = {
          default = flake-utils.lib.mkApp { drv = dagrid-standalone; };
          dagrid-plugin = flake-utils.lib.mkApp { drv = dagrid-plugin; };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            rust-analyzer
            pipewire.jack
          ];

          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${with pkgs; pkgs.lib.makeLibraryPath [ libjack2 ]}";
        };
      }
    );
}
