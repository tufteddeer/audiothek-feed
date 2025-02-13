{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) {
          inherit system overlays;
        };

        toolchain = pkgs.rust-bin.selectLatestNightlyWith
          (toolchain: toolchain.default.override {
            extensions = [ "rust-src" ];
          });

        naersk' = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
        };


      in
      rec {

        nixosModules.audiothekfeed = import ./modules/audiothekfeed self;
        packages.backend = naersk'.buildPackage {
            src = ./.;
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            buildInputs = with pkgs; [
              openssl
              packages.frontend
            ];

            FRONTEND_DIR = "${packages.frontend}";


          };

        packages.frontend = pkgs.buildNpmPackage
          rec {
            name = "audiothek-feed-frontend";
            src = ./frontend;
            npmDepsHash = "sha256-GUQcOZFI7pSt1RD6KQIyUqUttjZhjofDLICn9cE/Vxg=";
            installPhase =
              ''
                mkdir $out
                cp -r *.css *.html $out
              '';
            npmBuild = "npm run build";
          };
        # For `nix build` & `nix run`:
        defaultPackage = packages.backend;

        # For `nix develop` (optional, can be skipped):
        devShell = pkgs.mkShell
          {
            nativeBuildInputs = with pkgs; [
              toolchain
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl
            ];
            FRONTEND_DIR = "../frontend";

          };

        packages.image = pkgs.dockerTools.buildImage {
          name = "audiothek-feed";
          tag = "0.1.0";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ pkgs.cacert ];
          };

          config = {
            Cmd = [ "${packages.backend}/bin/audiothek-feed" ];

            ExposedPorts = {
              "3000/tcp" = { };
            };
          };

          created = "now";
        };
      }
    );
}
