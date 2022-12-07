{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib.toolchains) rustToolchain;
  l = nixpkgs.lib // builtins;

  mkEnv = env: l.mapAttrsToList (name: value: {inherit name value;}) env;

  catalystCore = {...}: {
    name = nixpkgs.lib.mkForce "Catalyst Core";
    env = mkEnv {
      PROTOC = "${nixpkgs.protobuf}/bin/protoc";
      PROTOC_INCLUDE = "${nixpkgs.protobuf}/include";
    };
    nixago = [
      cell.configs.lefthook
      cell.configs.prettier
      cell.configs.treefmt
    ];
    packages = with nixpkgs; [
      rustToolchain
      pkg-config
      openssl
      protobuf
      uniffi-bindgen
      postgresql
      diesel-cli
      cargo-insta # snapshot testing lib
    ];
  };
in
  l.mapAttrs (_: std.lib.dev.mkShell) rec {
    dev = {...}: {
      imports = [
        catalystCore
      ];
    };
    ops = {...}: {
      imports = [
        catalystCore
      ];
    };
    default = dev;
  }
