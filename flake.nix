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
    voting-tools.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {std, ...} @ inputs:
    std.growOn
    {
      inherit inputs;
      cellsFrom = ./nix;

      cellBlocks = [
        (std.blockTypes.containers "containers")
        (std.blockTypes.devshells "devshells")
        (std.blockTypes.functions "constants")
        (std.blockTypes.functions "lib")
        (std.blockTypes.functions "toolchains")
        (std.blockTypes.installables "packages" {ci.build = true;})
        (std.blockTypes.nixago "configs")
        (std.blockTypes.runnables "operables")
      ];
    }
    {
      devShells = std.harvest inputs.self ["automation" "devshells"];
      packages = std.harvest inputs.self [
        ["artifacts" "packages"]
        ["catalyst-toolbox" "packages"]
        ["chain-libs" "packages"]
        ["chain-wallet-libs" "packages"]
        ["jormungandr" "packages"]
        ["jortestkit" "packages"]
        ["vit-servicing-station" "packages"]
        ["vit-testing" "packages"]
        ["voting-tools" "packages"]
        ["voting-tools-rs" "packages"]
      ];
      containers = std.harvest inputs.self [
        ["jormungandr" "containers"]
        ["vit-servicing-station" "containers"]
        ["vit-testing" "containers"]
      ];
    };

  nixConfig = {
    extra-substituters = [
      #"https://hydra.iohk.io"
      "https://iog-gov-nix.s3.eu-central-1.amazonaws.com"
    ];
    extra-trusted-public-keys = [
      "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ="
      "gov:uG8+LG8RqFGScUmOrDkGb4VCbtNhChbnycVnxZxb8AY="
    ];
    allow-import-from-derivation = "true";
  };
}
