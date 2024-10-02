{ inputs, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
  ];
  perSystem =
    {
      config,
      self',
      pkgs,
      lib,
      ...
    }:
    {
      rust-project.crates."gitwatch-rs".crane.args = {
        buildInputs = with pkgs; [
          openssl
        ];
      };
      packages.default = self'.packages.gitwatch-rs;
    };
}
