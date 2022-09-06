# provides the 2.0 version of the diesel cli

{ pkgs }:

pkgs.diesel-cli.overrideAttrs {
  version = "2.0.0";

  src = fetchCrate {
    inherit version;
    crateName = "diesel_cli";
    sha256 = "sha256-mRdDc4fHMkwkszY+2l8z1RSNMEQnrWI5/Y0Y2W+guQE=";
  };
}
