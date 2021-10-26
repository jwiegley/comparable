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
  pname = "delta";
  version = "0.0.1";

  src = ./.;

  cargoSha256 = "1wzifsf7sazf6a9hn2ia4fprd1dxg02cnynpfy8ca7jrw5g9vnxm";

  cargoBuildFlags = [];

  nativeBuildInputs = [ rls rustfmt clippy pkg-config ];
  buildInputs = [ openssl protobuf ]
    ++ (lib.optional stdenv.isDarwin darwin.apple_sdk.frameworks.Security);

  registry = "file://local-registry";

  meta = with lib; {
    description = "Differencing data structures to improve testing";
    homepage = https://github.com/jwiegley/delta;
    license = licenses.mit;
    maintainers = [ maintainers.jwiegley ];
    platforms = platforms.all;
  };
}
