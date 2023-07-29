{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";

    nix-npm-buildpackage.url = "github:serokell/nix-npm-buildpackage";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, rust-overlay, nix-npm-buildpackage }:
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
        packages.backend = naersk'.buildPackage
          {
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

        packages.frontend = nix-npm-buildpackage.legacyPackages.x86_64-linux.buildNpmPackage
          rec {
            src = ./frontend;
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
      }
    );
}
