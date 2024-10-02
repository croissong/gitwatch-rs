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
      rust-project.src = lib.cleanSourceWith {
        src = inputs.self;
        filter =
          path: type:
          config.rust-project.crane-lib.filterCargoSources path type
          || "${inputs.self}/docs/gitwatch.1" == path;
      };

      rust-project.crates."gitwatch-rs" = {
        crane.args = {
          buildInputs = with pkgs; [
            openssl
          ];
        };

      };
      packages.default = self'.packages.gitwatch-rs.overrideAttrs (oa: {
        nativeBuildInputs = oa.nativeBuildInputs ++ [ pkgs.installShellFiles ];
        postInstall = ''
          installShellCompletion --cmd gitwatch \
            --bash <($out/bin/gitwatch completion bash) \
            --fish <($out/bin/gitwatch completion fish) \
            --zsh <($out/bin/gitwatch completion zsh)

          installManPage docs/gitwatch.1
        '';

        meta = {
          description = "Watch a Git repository and automatically commit changes";
          mainProgram = "gitwatch";
        };
      });
    };
}
