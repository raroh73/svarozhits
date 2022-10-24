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

        toolchain = fenix.packages.${system}.stable.toolchain;

        naerskBuilder = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };
      in
      rec {
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              markdownlint.enable = true;
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };
          };
        };

        devShells.default = pkgs.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
          nativeBuildInputs = [ toolchain ] ++ [ pkgs.sqlx-cli ];
        };

        packages.default = packages.x86_64-unknown-linux-gnu;

        packages.x86_64-unknown-linux-gnu = naerskBuilder.buildPackage {
          src = ./.;
          doCheck = true;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";
        };

        packages.docker-image-linux-amd64 = pkgs.dockerTools.buildLayeredImage {
          name = "ghcr.io/raroh73/svarozhits";
          tag = "latest";
          config = {
            Entrypoint = [ "${packages.x86_64-unknown-linux-gnu}/bin/svarozhits" ];
            ExposedPorts = {
              "3000/tcp" = { };
            };
            WorkingDir = "/svarozhits";
          };
        };
      }
    );
}
