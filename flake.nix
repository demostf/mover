{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in
        rec {
          # `nix build`
          packages.demomover = naersk-lib.buildPackage {
            pname = "demomover";
            root = ./.;
          };
          defaultPackage = packages.demomover;

          # `nix run`
          apps.hello-world = flake-utils.lib.mkApp {
            drv = packages.demomover;
          };
          defaultApp = apps.demomover;

          # `nix develop`
          devShell = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [ rustc cargo bacon ];
          };
        }
    );
}
