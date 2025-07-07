{inputs, ...}: {
  imports = [
    (inputs.git-hooks + /flake-module.nix)
  ];
  perSystem = {
    config,
    self',
    pkgs,
    lib,
    ...
  }: {
    formatter = pkgs.alejandra;
    pre-commit.settings = {
      hooks = {
        alejandra.enable = true;
        rustfmt.enable = true;
      };
    };
  };
}
