{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [
          (import ./overlay.nix)
        ];
        pkgs = (import nixpkgs) {
          inherit system overlays;
        };
      in rec {
        packages = rec {
          demostf-mover = pkgs.demostf-mover;
          default = demostf-mover;
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo bacon clippy cargo-audit cargo-edit];
        };
      }
    )
    // {
      overlays.default = import ./overlay.nix;
      nixosModules.default = {
        pkgs,
        config,
        lib,
        ...
      }: {
        imports = [./module.nix];
        config = lib.mkIf config.services.demostf-mover.enable {
          nixpkgs.overlays = [self.overlays.default];
          services.demostf-mover.package = lib.mkDefault pkgs.demostf-mover;
        };
      };
    };
}
