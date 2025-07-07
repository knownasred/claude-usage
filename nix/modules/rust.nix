{
  inputs,
  self,
  ...
}: {
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem = {
    config,
    self',
    pkgs,
    lib,
    ...
  }: {
    rust-project = {
      crates = {
        claude-usage.path = (inputs.self) + /crates/claude-usage;
      };
    };

    packages = {
      default = self'.packages.claude-usage;
    };
  };
}
