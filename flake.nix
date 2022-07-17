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
    )
    // {
      nixosModule = {
        config,
        lib,
        pkgs,
        ...
      }:
        with lib; let
          cfg = config.services.demosmover;
        in {
          options.services.demosmover = {
            enable = mkEnableOption "Enables the demos mover service";

            source = mkOption {
              type = types.str;
              description = "source directory";
            };
            target = mkOption {
              type = types.str;
              description = "target directory";
            };
            sourceBackend = mkOption {
              type = types.str;
              description = "source backend";
            };
            targetBackend = mkOption {
              type = types.str;
              description = "target backend";
            };
            keyFile = mkOption {
              type = types.str;
              description = "file containing the api key";
            };
            age = mkOption {
              type = types.int;
              default = 78894000;
              description = "age of files to move in secconds";
            };
            logLevel = mkOption {
              type = types.str;
              default = "INFO";
              description = "log level";
            };
            user = mkOption {
              type = types.str;
              description = "user that owns the demos";
            };
            interval = mkOption {
              type = types.str;
              default = "*:0/10";
              description = "Interval to run the service";
            };
          };

          config = mkIf cfg.enable {
            systemd.services.demosmover = let
              pkg = self.defaultPackage.${pkgs.system};
            in {
              script = "${pkg}/bin/mover";
              description = "Move demos for demos.tf";

              environment = {
                SOURCE_ROOT = cfg.source;
                TARGET_ROOT = cfg.target;
                SOURCE_BACKEND = cfg.sourceBackend;
                TARGET_BACKEND = cfg.targetBackend;
                AGE = toString cfg.age;
                RUST_LOG = cfg.logLevel;
              };

              serviceConfig = {
                EnvironmentFile = cfg.keyFile;
                ReadWritePaths = [cfg.source cfg.target];
                Restart = "on-failure";
                User = cfg.user;
                PrivateTmp = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                NoNewPrivileges = true;
                PrivateDevices = true;
                ProtectClock = true;
                CapabilityBoundingSet = true;
                ProtectKernelLogs = true;
                ProtectControlGroups = true;
                SystemCallArchitectures = "native";
                ProtectKernelModules = true;
                RestrictNamespaces = true;
                MemoryDenyWriteExecute = true;
                ProtectHostname = true;
                LockPersonality = true;
                ProtectKernelTunables = true;
                RestrictAddressFamilies = "AF_INET AF_INET6";
                RestrictRealtime = true;
                ProtectProc = "noaccess";
                SystemCallFilter = ["@system-service" "~@resources" "~@privileged"];
                IPAddressDeny = "localhost link-local multicast";
              };
            };

            systemd.timers.demosmover = {
              enable = true;
              description = "Move demos for demos.tf";
              wantedBy = ["multi-user.target"];
              timerConfig = {
                OnCalendar = "*:0/10";
              };
            };
          };
        };
    };
}
