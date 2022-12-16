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
    env = with nixpkgs;
      mkEnv {
        OPENSSL_NO_VENDOR = 1;
        OPENSSL_DIR = "${l.getDev openssl}";
        OPENSSL_LIB_DIR = "${l.getLib openssl}/lib";
        PROTOC = "${protobuf}/bin/protoc";
        PROTOC_INCLUDE = "${protobuf}/include";
      };
    nixago = [
      cell.configs.lefthook
      cell.configs.prettier
      cell.configs.treefmt
    ];
    packages = with nixpkgs; [
      gcc
      rustToolchain
      pkg-config
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
