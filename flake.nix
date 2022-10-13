{
  description = "Svarozhits";

  inputs = {
    fenix =
      {
        url = "github:nix-community/fenix";
        inputs.nixpkgs.follows = "nixpkgs";
      };
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, fenix, flake-utils, naersk, nixpkgs, pre-commit-hooks }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        toolchain = with fenix.packages.${system};
          combine [
            default.rustc
            default.cargo
            default.clippy
            default.rustfmt
            targets.x86_64-unknown-linux-gnu.latest.rust-std
            targets.aarch64-unknown-linux-gnu.latest.rust-std
          ];

        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };

        naerskBuildPackage = target: args:
          naersk'.buildPackage (
            args
            // { CARGO_BUILD_TARGET = target; }
            // cargoConfig
          );

        cargoConfig = {
          CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
        };

      in
      rec {
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              #cargo-check.enable = true;
              #clippy.enable = true;
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };
          };
        };

        packages.default = packages.x86_64-unknown-linux-gnu;

        packages.x86_64-unknown-linux-gnu = naerskBuildPackage "x86_64-unknown-linux-gnu" {
          src = ./.;
          doCheck = true;
        };

        packages.aarch64-unknown-linux-gnu = naerskBuildPackage "aarch64-unknown-linux-gnu" {
          src = ./.;
        };

        packages.docker-image-amd64 = pkgs.dockerTools.buildLayeredImage {
          name = "ghcr.io/raroh73/svarozhits";
          tag = "amd64";
          config = {
            Cmd = [ "${packages.x86_64-unknown-linux-gnu}/bin/svarozhits" ];
          };
        };

        packages.docker-image-arm64 = pkgs.pkgsCross.aarch64-multiplatform.dockerTools.buildLayeredImage {
          name = "ghcr.io/raroh73/svarozhits";
          tag = "arm64";
          config = {
            Cmd = [ "${packages.aarch64-unknown-linux-gnu}/bin/svarozhits" ];
          };
        };

        devShells.default = pkgs.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
          nativeBuildInputs = [ toolchain ];
        };
      }
    );
}
