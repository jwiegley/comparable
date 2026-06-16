{
  description = "comparable - deterministic change detection for data structures, oriented toward testing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        inherit (pkgs) lib;

        # Stable toolchain: build, test, clippy, rustfmt, docs and coverage.
        # llvm-tools-preview is what cargo-llvm-cov needs.
        rustStable = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "llvm-tools-preview" ];
        };

        # Nightly toolchain for Miri and cargo-fuzz.  selectLatestNightlyWith
        # picks the most recent nightly that actually ships every component we
        # ask for, so the flake never breaks on a day Miri is missing.
        rustNightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {
            extensions = [ "rust-src" "miri" "llvm-tools-preview" ];
          });

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustStable;
          rustc = rustStable;
        };

        commonBuildInputs = [ pkgs.openssl ]
          ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
        commonNativeBuildInputs = [ pkgs.pkg-config ];

        # The published library.  Building straight from Cargo.lock means there
        # is no vendored hash to keep in sync, and doCheck runs the whole test
        # suite (including doctests) as part of `nix build`.
        comparable = rustPlatform.buildRustPackage {
          pname = "comparable";
          version = "0.5.6";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = commonBuildInputs;
          cargoBuildFlags = [ "--workspace" ];
          cargoTestFlags = [ "--workspace" ];
          doCheck = true;
          meta = with lib; {
            description = "Differencing data structures to improve testing";
            homepage = "https://github.com/jwiegley/comparable";
            license = with licenses; [ mit asl20 ];
            maintainers = [ maintainers.jwiegley ];
          };
        };

        # The fmt/clippy/docs checks reuse the package's vendored dependencies
        # and toolchain; only the build phase differs.
        mkPhaseCheck = name: env: phase:
          comparable.overrideAttrs (_: {
            pname = "comparable-${name}";
            buildPhase = phase;
            doCheck = false;
            installPhase = "touch $out";
          } // env);

        cargoTools = with pkgs; [
          cargo-nextest
          cargo-llvm-cov
          cargo-audit
          cargo-expand
          critcmp
        ];
        lintTools = with pkgs; [ nixpkgs-fmt statix shfmt shellcheck ];
        miscTools = with pkgs; [ jq git gnumake lefthook ];
      in
      {
        packages.default = comparable;
        packages.comparable = comparable;

        # `nix flake check` builds the library, runs every test, and verifies
        # formatting, clippy (warnings as errors) and a warning-free doc build.
        checks = {
          build = comparable;
          clippy = mkPhaseCheck "clippy" { }
            "cargo clippy --workspace --all-targets -- -D warnings";
          fmt = mkPhaseCheck "fmt" { }
            "cargo fmt --all -- --check";
          docs = mkPhaseCheck "docs" { RUSTDOCFLAGS = "-D warnings"; }
            "cargo doc --workspace --no-deps";
        };

        formatter = pkgs.nixpkgs-fmt;

        # Default shell: everything except Miri and fuzzing.
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = [ rustStable ] ++ cargoTools ++ lintTools ++ miscTools
            ++ commonBuildInputs;
          shellHook = ''
            echo "comparable dev shell -- run 'make help' to list targets."
            echo "Miri and fuzzing live in the nightly shell: nix develop .#nightly"
          '';
        };

        # Nightly shell for `make miri` and `make fuzz`.
        devShells.nightly = pkgs.mkShell {
          nativeBuildInputs = commonNativeBuildInputs;
          buildInputs = [ rustNightly pkgs.cargo-fuzz ] ++ miscTools
            ++ commonBuildInputs;
        };
      }
    );
}
