{ inputs, ... }:
{
  perSystem =
    {
      config,
      self',
      pkgs,
      lib,
      ...
    }:
    let
      libPath = lib.makeLibraryPath [
        pkgs.openssl
      ];
    in
    {

      devShells.default = pkgs.mkShell {
        name = "gitwatch-rs-shell";
        inputsFrom = [
          self'.devShells.rust
        ];
        packages = with pkgs; [
          bacon
          cargo-udeps
          cargo-tarpaulin
          cargo-nextest
          clang
        ];

        LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${libPath}";
      };
    };
}
