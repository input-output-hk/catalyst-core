{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (cell.toolchains) naersk;
  l = nixpkgs.lib // builtins;
in rec {
  readTOML = file: l.fromTOML (l.readFile file);
  mkPackage = {
    pkgPath,
    cargoOptions ? [],
    nativeBuildInputs ? [],
  }: let
    pkgCargo = readTOML (pkgPath + "/Cargo.toml");
    inherit (pkgCargo.package) name version;

    cargoOptions' = ["--package" "${name}"] ++ cargoOptions;
    nativeBuildInputs' = with nixpkgs;
      [
        pkg-config
        protobuf
        rustfmt
      ]
      # TODO: move this
      ++ (pkgs.lib.optionals (name == "voting_tools_rs" || name == "vit-servicing-station-server") [
        postgresql
      ])
      ++ nativeBuildInputs;
  in
    naersk.buildPackage {
      inherit name version;

      root = inputs.self;
      nativeBuildInputs = nativeBuildInputs';

      cargoBuildOptions = x: x ++ cargoOptions';
      cargoTestOptions = x: x ++ cargoOptions';

      PROTOC = "${nixpkgs.protobuf}/bin/protoc";
      PROTOC_INCLUDE = "${nixpkgs.protobuf}/include";

      buildInputs = with nixpkgs; [
        openssl
      ];
    };
  wrap = pkg: deps: let
    name = pkg.name;
  in
    nixpkgs.runCommand "wrapped-${name}" {nativeBuildInputs = [nixpkgs.makeWrapper];} ''
      mkdir -p $out/bin
      ln -s ${pkg}/bin/${name} $out/bin/${name}
      wrapProgram $out/bin/${name} --prefix PATH : ${l.makeBinPath deps}
    '';
}
