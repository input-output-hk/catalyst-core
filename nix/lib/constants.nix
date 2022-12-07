{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs;
  l = nixpkgs.lib // builtins;

  envs = [
    "dev"
    "signoff"
    "perf"
    "dryrun"
    "prod"
  ];
  events = [
    "fund10"
    "sve1"
    "sve2"
  ];

  mkNamespaces = event: l.map (env: "${event}-${env}") envs;
in rec {
  inherit envs events;
  namespaces = l.flatten (l.map (event: mkNamespaces event) events);
  registry = "registry.ci.iog.io";

  mapToNamespaces = {
    prefix ? "",
    suffix ? "",
  }: fn:
    l.listToAttrs (
      l.map
      (
        namespace: {
          name =
            if prefix != ""
            then
              if suffix != ""
              then "${prefix}-${namespace}-${suffix}"
              else "${prefix}-${namespace}"
            else "${namespace}";
          value = fn namespace;
        }
      )
      namespaces
    );
}
