{ rev    ? "98747f27ecfee70c8c97b195cbb94df80a074dda"
, sha256 ? "04ss525ns5qqlggrdhvc6y4hqmshylda9yd0y99ddliyn15wmf27"
, pkgs   ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
    inherit sha256; }) {
    config.allowUnfree = true;
    config.allowBroken = false;
  }
}:

with pkgs; rustPlatform.buildRustPackage rec {
  pname = "comparable";
  version = "0.5.1";

  src = ./.;

  cargoSha256 = "sha256-f/s/8gT3/8a/kztn2Dmbl/rCmwYfQqddH3Q9u2/+7JE=";

  cargoBuildFlags = [];

  nativeBuildInputs = [ rls rustfmt clippy pkg-config cargo-expand ];
  buildInputs = [ openssl protobuf ]
    ++ (lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security);

  RUSTC_BOOTSTRAP = 1;

  meta = with lib; {
    description = "Differencing data structures to improve testing";
    homepage = https://github.com/jwiegley/comparable;
    license = licenses.mit;
    maintainers = [ maintainers.jwiegley ];
    platforms = platforms.all;
  };
}
