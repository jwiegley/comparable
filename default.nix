{ rev    ? "8e1eab9eae4278c9bb1dcae426848a581943db5a"
, sha256 ? "0jf4rccc0z9in1iahw13i5wl93gbp1x6mkjd3qivjg97ms9qw3l0"
, pkgs   ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
    inherit sha256; }) {
    config.allowUnfree = true;
    config.allowBroken = false;
  }
}:

with pkgs; rustPlatform.buildRustPackage rec {
  pname = "comparable";
  version = "0.4.0";

  src = ./.;

  cargoSha256 = "0m3b32zcby7l6j8x0p6g32afpknjscx826apbh1jk8r2pinjcmn8";

  cargoBuildFlags = [];

  nativeBuildInputs = [ rls rustfmt clippy pkg-config cargo-expand ];
  buildInputs = [ openssl protobuf ]
    ++ (lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security);

  registry = "file://local-registry";

  RUSTC_BOOTSTRAP = 1;

  meta = with lib; {
    description = "Differencing data structures to improve testing";
    homepage = https://github.com/jwiegley/comparable;
    license = licenses.mit;
    maintainers = [ maintainers.jwiegley ];
    platforms = platforms.all;
  };
}
