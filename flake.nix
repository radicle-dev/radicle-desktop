{
  description = "Radicle Desktop App";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    heartwood = {
      url = "git+https://seed.radicle.xyz/z3gqcJUoA1n9HaHKufZs5FCSGazv5.git?ref=refs/namespaces/z6MksFqXN3Yhqk8pTJdUGLwATkRfQvwZXPqR2qMEhbS9wzpT/refs/tags/v1.1.0";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    heartwood,
    ...
  }:
    (flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };
    in {

      checks = {
        radicle-desktop = self.packages.${system}.radicle-desktop.overrideAttrs ({ doCheck = true; });
      };

      devShells.default = pkgs.mkShell {
        name = "radicle-desktop-env";
        inputsFrom = [ self.checks.${system}.radicle-desktop ];
        nativeBuildInputs = with pkgs; [
          cargo-watch
          cargo-nextest
          ripgrep
          rust-analyzer
        ];
        env = self.checks.${system}.radicle-desktop.env // {
        };
      };

      packages = {
        default = self.packages.${system}.radicle-desktop;
        twemoji-assets = pkgs.fetchFromGitHub {
          owner = "twitter";
          repo = "twemoji";
          rev = "v14.0.2";
          hash = "sha256-YoOnZ5uVukzi/6bLi22Y8U5TpplPzB7ji42l+/ys5xI=";
        };

        radicle-desktop = pkgs.callPackage (
          {
            lib,
            importNpmLock,
            rust-bin,
            makeRustPlatform,
            cargo-tauri,
            nodejs,
            pkg-config,
            wrapGAppsHook4,
            glib,
            gtk3,
            libsoup_3,
            openssl,
            webkitgtk_4_1,
            git,
            openssh,
          }: let
            rTc = rust-bin.fromRustupToolchainFile ./rust-toolchain;
            rustPlatform = makeRustPlatform {
              cargo = rTc;
              rustc = rTc;
            };
          in rustPlatform.buildRustPackage rec {
            pname = "radicle-desktop";
            inherit (with builtins; (fromJSON (readFile ./package.json))) version;

            src = ./.;

            cargoDeps = rustPlatform.importCargoLock { 
              lockFile = ./Cargo.lock; 
              outputHashes = {
                "radicle-0.14.0" = "sha256-F7pJ+yLhlRXg03A+pNXwsqNSOG3qJs6bEO9YUUXs4f0=";
              };
            };

            npmDeps = importNpmLock {
              inherit version;
              pname = pname + "-npm-deps";
              npmRoot = ./.;
            };

            nativeBuildInputs = [
              cargo-tauri.hook
              nodejs
              importNpmLock.npmConfigHook
              pkg-config
              wrapGAppsHook4
            ];

            buildInputs = [
              glib gtk3 libsoup_3 openssl webkitgtk_4_1
            ];

            postPatch = ''
              patchShebangs scripts/copy-katex-assets scripts/check-js scripts/check-rs
              mkdir -p public/twemoji
              cp -t public/twemoji -r -- ${self.packages.${system}.twemoji-assets}/assets/svg/*
              : >scripts/install-twemoji-assets
            '';

            doCheck = false;
            nativeCheckInputs = [ git openssh ];

            env = {
              HW_RELEASE = "nix-" + (heartwood.shortRev or "unknown-ref");
              PLAYWRIGHT_BROWSERS_PATH = pkgs.playwright-driver.browsers;
              PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = 1;
              PLAYWRIGHT_SKIP_VALIDATE_HOST_REQUIREMENTS = true;
              RUST_SRC_PATH = "${rTc}/lib/rustlib/src/rust/library";
            } // (
              if self ? rev || self ? dirtyRev
              then {
                GIT_HEAD = self.rev or self.dirtyRev;
              }
              else {}
            );

            preCheck = ''
              export RAD_HOME="$PWD/_rad-home"
              export RAD_PASSPHRASE=""
              rad auth --alias test
              bins="tests/tmp/bin/heartwood/$HW_RELEASE"
              mkdir -p "$bins"
              cp -t "$bins" -- ${heartwood.packages.${system}.radicle}/bin/*
              printf "$HW_RELEASE" >tests/support/heartwood-release
            '';

            checkPhase = ''
              npm run build:http
              npm run test:unit
              scripts/check-js
              scripts/check-rs
            '';

            passthru.env = env;
            meta = {
              description = "Radicle Desktop App";
              license = lib.licenses.gpl3;
              maintainers = [ ];
            };
          }) {};
      };
    }));
}
