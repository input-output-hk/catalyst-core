{
  description = "Governance Voting Center";
  inputs = {
    ## Nixpkgs ##
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    ## Std ##
    std.url = "github:divnix/std";
    std.inputs.nixpkgs.follows = "nixpkgs";

    # Rust overlay
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

    # Naersk
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";

    # Cardano
    cardano-node.url = "github:input-output-hk/cardano-node/1.33.0";
    cardano-node.inputs.nixpkgs.follows = "nixpkgs";

    # Voting Tools
    voting-tools.url = "github:input-output-hk/voting-tools";
  };

  outputs = {std, ...} @ inputs:
    std.growOn
    {
      inherit inputs;
      cellsFrom = ./nix;

      cellBlocks = [
        (std.blockTypes.containers "containers" {ci.publish = true;})
        (std.blockTypes.devshells "devshells" {ci.build = true;})
        (std.blockTypes.functions "constants")
        (std.blockTypes.functions "lib")
        (std.blockTypes.functions "toolchains")
        (std.blockTypes.installables "libraries")
        (std.blockTypes.installables "packages" {ci.build = true;})
        (std.blockTypes.nixago "configs")
        (std.blockTypes.runnables "operables")
      ];
    }
    {
      devShells = std.harvest inputs.self ["automation" "devshells"];
      containers = std.harvest inputs.self [
        ["jormungandr" "containers"]
        ["vit-servicing-station" "containers"]
        ["vit-testing" "containers"]
      ];
      libraries = std.harvest inputs.self [
        ["catalyst-toolbox" "libraries"]
        ["chain-libs" "libraries"]
        ["chain-wallet-libs" "libraries"]
        ["jormungandr" "libraries"]
        ["jortestkit" "libraries"]
        ["vit-servicing-station" "libraries"]
        ["vit-testing" "libraries"]
      ];
      packages = std.harvest inputs.self [
        ["catalyst-toolbox" "packages"]
        ["jormungandr" "packages"]
        ["vit-servicing-station" "packages"]
        ["vit-testing" "packages"]
        ["voting-tools" "packages"]
        ["voting-tools-rs" "packages"]
      ];
    };

  nixConfig = {
    extra-substituters = [
      "https://cache.iog.io"
      "https://iog-catalyst-nix.s3.eu-central-1.amazonaws.com"
    ];
    extra-trusted-public-keys = [
      "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="
      "catalyst:kNW0n7ijUJDvu4BrpqC3j54rgoHNccXx7ABuVzuL9WM="
    ];
    allow-import-from-derivation = "true";
  };
}
