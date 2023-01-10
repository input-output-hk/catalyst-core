{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants;
  inherit (cell.toolchains) naersk rustToolchain;
  l = nixpkgs.lib // builtins;
in rec {
  readTOML = file: l.fromTOML (l.readFile file);

  # Combines root + path1 + path2 and then resolves any relative references
  # between them (i.e. root/foo/bar/../baz -> root/foo/baz)
  reifyPath = root: path1: path2: let
    absPath = l.toPath (root + path1 + "/${path2}");
    relPath = l.replaceStrings [root] [""] absPath;
  in
    relPath;

  # Given the path to the root of a cargo project, recursively fetches all local
  # dependencies (those specified by `path = ...`) and returns a list of their
  # deduplicated relative paths from the root.
  recurseFetchDeps = path: let
    toml = readTOML (inputs.self + "/src/" + path + "/Cargo.toml");
    deps = l.filterAttrs (k: v: v ? path) (
      if toml ? dependencies
      then toml.dependencies
      else {}
    );
    paths =
      l.mapAttrsToList (
        k: v:
          reifyPath (inputs.self + "/src/") path v.path
      )
      deps;
    allPaths = (l.map (p: recurseFetchDeps p) paths) ++ [path];
  in
    l.unique (l.flatten allPaths);

  # Recreates a cargo workspace by accepting a partial workspace representation
  # (src) and then initializing all workspace members that are not present in
  # the partial workspace.
  #
  # This is required in order to limit the input when compiling workspace
  # members. By only including required dependencies, and initializing the rest
  # to a known state, changes to derivation inputs can be reduced.
  mkDummySrc = members: src:
    nixpkgs.runCommand "source" {}
    ''
      cp -r ${src} tmp
      chmod -R 0744 tmp

      for c in ${l.concatStringsSep " " members}
      do
        if [[ ! -f tmp/$c/Cargo.toml ]]; then
          mkdir -p "tmp/$c"
          ${rustToolchain}/bin/cargo init tmp/$c
        fi
      done

      cp -r tmp $out
    '';

  # Given the path to a cargo project (relative from src/), returns a derivation
  # for building the project that utilizes the workspace lockfile but also
  # filters the inputs to only include required dependencies. This improves
  # caching by reducing the amount the generated output hash changes.
  mkPackage = {
    pkgPath,
    cargoOptions ? [],
    nativeBuildInputs ? [],
  }: let
    rootPkgCargo = readTOML (inputs.self + "/Cargo.toml");
    pkgCargo = readTOML (inputs.self + "/src/" + pkgPath + "/Cargo.toml");
    inherit (pkgCargo.package) name version;

    deps = l.map (d: "src/${d}") (recurseFetchDeps pkgPath);
    filteredSrc = std.incl inputs.self ([
        "Cargo.toml"
        "Cargo.lock"
      ]
      ++ deps);

    nativeBuildInputs' = with nixpkgs;
      [
        pkg-config
        protobuf
        rustfmt
        postgresql.lib
      ]
      ++ nativeBuildInputs;
  in
    naersk.buildPackage {
      inherit name version;

      # We have to invoke cargo from within the member's directory
      preBuild = "cd src/${pkgPath}";

      # The output artifacts are stored in the workspace root, failing to change
      # back will result in naesrk failing to find the artifacts.
      postBuild = "cd -";

      root = inputs.self;
      src = mkDummySrc rootPkgCargo.workspace.members filteredSrc;

      nativeBuildInputs = nativeBuildInputs';

      cargoBuildOptions = x: x ++ cargoOptions;
      cargoTestOptions = x: x ++ cargoOptions;

      PROTOC = "${nixpkgs.protobuf}/bin/protoc";
      PROTOC_INCLUDE = "${nixpkgs.protobuf}/include";

      buildInputs = with nixpkgs; [
        openssl
      ];
    };

  # Maps a function to all possible namespaces, returning results of the
  # function calls as an attribute set where the key is `{service}-{namespace}`
  # and the value is the function result.
  mapToNamespaces = service: fn:
    l.listToAttrs (
      l.map
      (
        namespace: {
          name = "${service}-${namespace}";
          value = fn namespace;
        }
      )
      constants.namespaces
    );
}
