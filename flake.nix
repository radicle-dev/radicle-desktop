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

        radicle-desktop = pkgs.callPackage ./nix/radicle-desktop.nix {
          inherit heartwood;
          inherit (self.packages.${system}) twemoji-assets;
        }
        // (if self ? rev || self ? dirtyRev then {
          GIT_HEAD = if self ? rev then self.rev else self.dirtyRev;
        } else {});
      };
    }));
}
