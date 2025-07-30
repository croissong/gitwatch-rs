{ ... }:
{
  perSystem =
    {
      self',
      pkgs,
      lib,
      ...
    }:
    let
      libPath =
        with pkgs;
        lib.makeLibraryPath [
          openssl
          libgit2
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
          cargo-edit
          cargo-udeps
          cargo-tarpaulin
          cargo-nextest
          clang
        ];

        LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${libPath}";
      };
    };
}
