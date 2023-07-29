flake: { config, lib, pkgs, ... }:

let
  inherit (lib)  mkEnableOption;

  cfg = config.services.audiothekfeed;

  package = flake.defaultPackage.${pkgs.stdenv.hostPlatform.system};
in
{

  options = {
    services.audiothekfeed = {
      enable = mkEnableOption ''
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

      after = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        User = "audiothekfeed";
        Group = "audiothekfeed";
        Restart = "always";
        ExecStart = "${lib.getBin package}/bin/audiothek-rss";
        StateDirectory = "audiothekfeed";
        StateDirectoryMode = "0750";

        # Hardening
        apabilityBoundingSet = [ "AF_NETLINK" "AF_INET" "AF_INET6" ];
        DeviceAllow = [ "/dev/stdin r" ];
        DevicePolicy = "strict";
        IPAddressAllow = "localhost";
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        fmemoryPrivateDevices = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectSystem = "strict";
        # ReadOnlyPaths = [ "/" ];
        RemoveIPC = true;
        RestrictAddressFamilies = [ "AF_NETLINK" "AF_INET" "AF_INET6" ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [ "@system-service" "~@privileged" "~@resources" "@pkey" ];
        # UMask = "0027";
      };

    };
  };
}
