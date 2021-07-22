{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:kreisys/flake-utils";
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.simpleFlake rec {
      inherit nixpkgs;
      systems = [ "x86_64-linux" "aarch64-linux" ];
      preOverlays = [ naersk overlay ];
      overlay = final: prev: {
        catalyst-toolbox = prev.naersk.buildPackage {
          inherit ((builtins.fromTOML
            (builtins.readFile (./Cargo.toml))).package)
            name version;
          root = ./.;
          nativeBuildInputs = with final; [ pkg-config protobuf rustfmt ];
          buildInputs = with final; [ openssl ];
          PROTOC = "${final.protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${final.protobuf}/include";
        };
      };
      packages = { catalyst-toolbox }@pkgs: pkgs;
      devShell =
        { mkShell, rustc, cargo, pkg-config, openssl, protobuf, rustfmt }:
        mkShell {
          PROTOC = "${protobuf}/bin/protoc";
          PROTOC_INCLUDE = "${protobuf}/include";
          buildInputs = [ rustc cargo pkg-config openssl protobuf rustfmt ];
        };
    };
}
