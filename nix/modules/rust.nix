{inputs, ...}: {
  imports = [
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
      # See /crates/*/crate.nix for the crate-specific Nix configuration
      crateNixFile = "crate.nix";
    };
  };
}
