{
  description = "Fina devShell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";

    buildbot-nix.url = "github:nix-community/buildbot-nix";
    buildbot-nix.inputs.nixpkgs.follows = "nixpkgs";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";

    crane.url = "github:ipetkov/crane";
  };

  nixConfig = {
    extra-trusted-public-keys = "fina.cachix.org-1:Xaf+3HF5Wffl4Gtpi68Yz/wRQw0bH8tNVTlMnkLSRQc=";
    extra-substituters = "https://fina.cachix.org";
  };

  outputs = inputs @ {
    flake-parts,
    nixpkgs,
    rust-overlay,
    crane,
    ...
  }:
    (flake-parts.lib.evalFlakeModule {inherit inputs;} (
      {
        lib,
        self,
        inputs,
        ...
      }: {
        imports = [
          # ./devshells.nix
        ];

        systems = [
          "x86_64-linux"
        ];

        perSystem = {
          self',
          system,
          pkgs,
          ...
        }: let
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default));

          # rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          # NB: we don't need to overlay our custom toolchain for the *entire*
          # pkgs (which would require rebuidling anything else which uses rust).
          # Instead, we just want to update the scope that crane will use by appending
          # our specific toolchain there.
          ## src = craneLib.cleanCargoSource ./.;
          unfilteredRoot = ./.; # The original, unfiltered source
          src = lib.fileset.toSource {
            root = unfilteredRoot;
            fileset = lib.fileset.unions [
              # Default files from crane (Rust and cargo files)
              (craneLib.fileset.commonCargoSources unfilteredRoot)
              # Include all the .sql migrations as well
              ./migrations
              ./.sqlx
              ./templates
            ];
          };

          # Common arguments can be set here to avoid repeating them later
          commonArgs = {
            inherit src;
            strictDeps = true;

            RUSTFLAGS = "-Z threads=4";
          };

          # Build *just* the cargo dependencies, so we can reuse
          # all of that work (e.g. via cachix) when running in CI
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          # Build the actual crate itself, reusing the dependency
          # artifacts from above.
          ## runs tests -> which will break currently due to network connectivity
          ## my-crate = craneLib.buildPackage (commonArgs
          ### my-crate = craneLib.cargoBuild (commonArgs
          ###   // {
          ###     inherit cargoArtifacts;
          ###   });
          my-crate = craneLib.buildPackage (commonArgs
            // {
              inherit cargoArtifacts;

              doCheck = false; # skip tests
            });
        in {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            # config.allowUnfree = true;
            overlays = [
              (import rust-overlay)
            ];
          };

          checks = {
            # Build the crate as part of `nix flake check` for convenience
            inherit my-crate;

            # Run clippy (and deny all warnings) on the crate source,
            # again, reusing the dependency artifacts from above.
            #
            # Note that this is done as a separate derivation so that
            # we can block the CI if there are issues here, but not
            # prevent downstream consumers from building our crate by itself.
            my-crate-clippy = craneLib.cargoClippy (commonArgs
              // {
                inherit cargoArtifacts;
                cargoClippyExtraArgs = "--all-targets -- --deny warnings";
              });
          };

          packages = {
            default = my-crate;

            cargoArtifacts = cargoArtifacts;
          };

          apps.default = {
            type = "app";
            program = my-crate;
          };

          devShells.default = craneLib.devShell {
            RUSTFLAGS = "-Zthreads=4";

            packages = with pkgs; [
              bacon
              cachix
              cargo-expand
              cargo-llvm-cov
              cargo-nextest
              jq
              postgresql
              python3
              svix-cli
              sqlx-cli
            ];
          };
        };
      }
    ))
    .config
    .flake;
}
