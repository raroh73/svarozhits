{
  description = "Svarozhits";

  inputs = {
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, advisory-db, crane, flake-utils, nixpkgs, rust-overlay }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        toolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "aarch64-unknown-linux-gnu" "x86_64-unknown-linux-gnu" ];
        };

        src = ./.;

        checksCraneLib = crane.lib.${system}.overrideToolchain toolchain;

        amd64CraneLib = (crane.mkLib (import nixpkgs {
          inherit system;
          crossSystem = "x86_64-linux";
        })).overrideToolchain toolchain;

        arm64CraneLib = (crane.mkLib (import nixpkgs {
          inherit system;
          crossSystem = "aarch64-linux";
        })).overrideToolchain toolchain;

        cargoArtifacts = checksCraneLib.buildDepsOnly {
          inherit src;
        };

        amd64Build =
          { stdenv }: amd64CraneLib.buildPackage {
            inherit src;
            doCheck = false;
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";
          };

        arm64Build =
          { stdenv }: arm64CraneLib.buildPackage {
            inherit src;
            doCheck = false;
            CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.pkgsCross.aarch64-multiplatform.stdenv.cc}/bin/aarch64-unknown-linux-gnu-gcc";
            HOST_CC = "${pkgs.stdenv.cc}/bin/gcc";
          };
      in
      rec {
        checks = {
          audit = checksCraneLib.cargoAudit {
            inherit src advisory-db;
          };
          clippy = checksCraneLib.cargoClippy {
            inherit cargoArtifacts src;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          };
          fmt = checksCraneLib.cargoFmt {
            inherit src;
          };
          test = checksCraneLib.cargoTest {
            inherit cargoArtifacts src;
          };
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ toolchain ] ++ [ pkgs.sqlx-cli ];
        };

        packages.default = packages.amd64-unknown-linux-gnu;

        packages.amd64-unknown-linux-gnu = pkgs.callPackage amd64Build { };

        packages.arm64-unknown-linux-gnu = pkgs.callPackage arm64Build { };

        packages.docker-image-linux-amd64 = pkgs.dockerTools.buildLayeredImage {
          name = "ghcr.io/raroh73/svarozhits";
          tag = "amd64";
          config = {
            Entrypoint = [ "${packages.amd64-unknown-linux-gnu}/bin/svarozhits" ];
            ExposedPorts = {
              "3000/tcp" = { };
            };
            WorkingDir = "/svarozhits";
          };
        };

        packages.docker-image-linux-arm64 = pkgs.pkgsCross.aarch64-multiplatform.dockerTools.buildLayeredImage {
          name = "ghcr.io/raroh73/svarozhits";
          tag = "arm64";
          config = {
            Entrypoint = [ "${packages.arm64-unknown-linux-gnu}/bin/svarozhits" ];
            ExposedPorts = {
              "3000/tcp" = { };
            };
            WorkingDir = "/svarozhits";
          };
        };
      }
    );
}
