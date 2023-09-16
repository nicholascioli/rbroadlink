{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        version = "0.4.0";

        buildExample = example: naersk-lib.buildPackage {
          inherit version;
          pname = example;
          src = ./.;

          # Enable the necessary features for the cli
          cargoBuildOptions = old: old ++ [
            "--features ${example}"
          ];

          # Only build the example
          overrideMain = old: {
            preConfigure = ''
              cargo_build_options="$cargo_build_options --example ${example}"
            '';
          };
        };
      in {
        packages = rec {
          default = rbroadlink-cli;
          rbroadlink-cli = buildExample "rbroadlink-cli";
          mqtt-broadlink = buildExample "mqtt-broadlink";

          container = pkgs.dockerTools.buildLayeredImage {
            name = "ghcr.io/nicholascioli/rbroadlink";
            tag = "latest";
            contents = [rbroadlink-cli mqtt-broadlink];
            config = {
              Entrypoint = [
                "${rbroadlink-cli}/bin/rbroadlink-cli"
              ];
              Env = [
                "RUST_LOG=info"
              ];
            };
          };
        };

        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy act ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      });
}
