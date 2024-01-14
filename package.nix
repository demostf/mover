{
  stdenv,
  rustPlatform,
  lib,
}: let
  inherit (lib.sources) sourceByRegex;
  src = sourceByRegex ./. ["Cargo.*" "(src)(/.*)?"];
in
  rustPlatform.buildRustPackage rec {
    pname = "demostf-mover";
    version = "0.1.0";

    inherit src;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };
  }
