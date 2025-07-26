{
  lib,
  pkgs,
  config,
  ...
}:
let
  inherit (lib)
    literalExpression
    mkOption
    ;
  mkService = name: cfg: {
    Unit.Description = "Gitwatch ${name}";
    Install.WantedBy = [ "default.target" ];

    Service = {
      Environment = [
        "PATH=${lib.makeBinPath (cfg.extraPackages)}"
      ];
      ExecStart =
        let
          args = cfg.args ++ [ cfg.repo_path ];
        in
        "${pkgs.gitwatch-rs}/bin/gitwatch watch ${lib.concatStringsSep " " args}";
    };
  };
in
{
  options.services.gitwatch = mkOption {
    type =
      with lib.types;
      attrsOf (submodule {
        options = {
          repo_path = mkOption {
            type = path;
            example = literalExpression ''\${config.home.homeDirectory}/notes/'';
            description = "The local repository path to watch";
          };

          args = mkOption {
            type = listOf str;
            default = [ ];
            example = [ "--log-level=debug" ];
            description = ''
              Additional arguments to pass to gitwatch watch command.
            '';
          };

          extraPackages = mkOption {
            type = listOf package;
            default = [ ];
            example = literalExpression "with pkgs; [ bash coreutils git aichat ]";
            description = ''
              Extra packages available to the commmit message script.
            '';
          };
        };
      });
  };

  config.systemd.user.services = lib.mapAttrs' (
    name: cfg: lib.nameValuePair "gitwatch-${name}" (mkService name cfg)
  ) config.services.gitwatch;
}
