flake: { config, lib, pkgs, ... }:

let
  inherit (builtins) toJSON removeAttrs;
  inherit (lib) filterAttrs types mkEnableOption mkOption mkRenamedOptionModule;
  inherit (lib.trivial) pipe;

  inherit (flake.packages.${pkgs.stdenv.hostPlatform.system}) defaultPackage;

  cfg = config.services.audiothekfeed;

in
{
  # imports = [
  #   (mkRenamedOptionModule [ "services" "foundryvtt" "hostname" ] [ "services" "foundryvtt" "hostName" ])
  # ];

  options = {
    services.audiothekfeed = {
      enable = mkEnableOption ''
        Foundry Virtual Tabletop: A standalone application for online tabletop role-playing games.
      '';
      };
      };

  config = lib.mkIf cfg.enable {
    users.users.audiothekfeed = {
      description = "Audiothek-Feed daemon user";
      isSystemUser = true;
      group = "audiothekfeed";
    };

    users.groups.audiothekfeed = { };

    systemd.services.audiothekfeed = {
      description = "Audiothek Atom feed provider";
      # documentation = [ "https://foundryvtt.com/kb/" ];

      after = [ "network-online.target" ];
      # wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        User = "audiothekfeed";
        Group = "audiothekfeed";
        Restart = "always";
        ExecStart = "${lib.getBin cfg.package}/bin/audiothek-rss";
        StateDirectory = "audiothekfeed";
        StateDirectoryMode = "0750";

        # Hardening
        # CapabilityBoundingSet = [ "AF_NETLINK" "AF_INET" "AF_INET6" ];
        # DeviceAllow = [ "/dev/stdin r" ];
        # DevicePolicy = "strict";
        # IPAddressAllow = "localhost";
        # LockPersonality = true;
        # # MemoryDenyWriteExecute = true;
        # NoNewPrivileges = true;
        # PrivateDevices = true;
        # PrivateTmp = true;
        # PrivateUsers = true;
        # ProtectClock = true;
        # ProtectControlGroups = true;
        # ProtectHome = true;
        # ProtectHostname = true;
        # ProtectKernelLogs = true;
        # ProtectKernelModules = true;
        # ProtectKernelTunables = true;
        # ProtectSystem = "strict";
        # ReadOnlyPaths = [ "/" ];
        # RemoveIPC = true;
        # RestrictAddressFamilies = [ "AF_NETLINK" "AF_INET" "AF_INET6" ];
        # RestrictNamespaces = true;
        # RestrictRealtime = true;
        # RestrictSUIDSGID = true;
        # SystemCallArchitectures = "native";
        # SystemCallFilter = [ "@system-service" "~@privileged" "~@resources" "@pkey" ];
        # UMask = "0027";
      };

      # preStart = ''
      #   installedConfigFile="${config.services.foundryvtt.dataDir}/Config/options.json"
      #   install -d -m750 ${config.services.foundryvtt.dataDir}/Config
      #   rm -f "$installedConfigFile" && install -m640 ${configFile} "$installedConfigFile"
      # '';
    };
  };
}
