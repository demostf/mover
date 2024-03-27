{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.demostf-mover;
  format = pkgs.formats.toml {};
  configFile = format.generate "demostf-mover.toml" {
    api = {
      url = cfg.api;
      key_file = "$CREDENTIALS_DIRECTORY/api_key";
    };
    source = {
      backend = cfg.sourceBackend;
      root = cfg.source;
    };
    target = {
      backend = cfg.targetBackend;
      root = cfg.target;
    };
    migrate = {
      age = cfg.age;
    };
  };
in {
  options.services.demostf-mover = {
    enable = mkEnableOption "Enables the demos mover service";

    api = mkOption {
      type = types.str;
      default = "https://api.demos.tf";
      description = "Api endpoint to migrate demos for";
    };
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

    package = mkOption {
      type = types.package;
      defaultText = literalExpression "pkgs.demostf-mover";
      description = "package to use";
    };
  };

  config = mkIf cfg.enable {
    systemd.services.demostf-mover = {
      description = "Move demos for demos.tf";

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/demostf-mover ${configFile}";
        ReadWritePaths = [cfg.source cfg.target];
        LoadCredential = [
          "api_key:${cfg.keyFile}"
        ];
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

    systemd.timers.demostf-mover = {
      enable = true;
      description = "Move demos for demos.tf";
      wantedBy = ["multi-user.target"];
      timerConfig = {
        OnCalendar = "*:0/10";
      };
    };
  };
}
