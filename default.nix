{ rev    ? "14ccaaedd95a488dd7ae142757884d8e125b3363"
, sha256 ? "1dvdpwdzkzr9pkvb7pby0aajgx7qv34qaxb1bjxx4dxi3aip9q5q"
, pkgs   ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
    inherit sha256; }) {
    config.allowUnfree = true;
    config.allowBroken = false;
  }
}:

with pkgs; rustPlatform.buildRustPackage rec {
  pname = "comparable";
  version = "0.5.4";

  src = ./.;

  cargoSha256 = "sha256-SZnvzTJZ1LJRqb5ssV2qQW5AlHhaENRE8L9ItjanHWQ=";

  cargoBuildFlags = [];

  nativeBuildInputs = [ rust-analyzer rustfmt clippy pkg-config cargo-expand ];
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
