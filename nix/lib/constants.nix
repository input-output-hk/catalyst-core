{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs;
  l = nixpkgs.lib // builtins;
in rec {
  # The current funding round we are in
  fundRound = 10;

  # The current SVE round we are in
  sveRound = 2;

  # List of target environments we generate artifacts for
  envs = [
    "dev"
    "signoff"
    "perf"
    "dryrun"
    "prod"
  ];

  # A list of all possible round/namespace combinations
  # fund10-dev, fund10-dryrun, sve2-signoff, sve2-prod, etc.
  namespaces = let
    events = [
      "fund${l.toString fundRound}"
      "sve${l.toString sveRound}"
    ];
    mkNamespaces = event: l.map (env: "${event}-${env}") envs;
  in
    l.flatten (l.map (event: mkNamespaces event) events);

  # The OCI registry we are pushing to
  registry = "332405224602.dkr.ecr.eu-central-1.amazonaws.com";
}
