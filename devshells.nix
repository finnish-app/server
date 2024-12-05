{
  perSystem = {
    inputs',
    self',
    pkgs,
    system,
    ...
  }: {
    devShells.default = let
      rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
    in
      pkgs.mkShellNoCC {
        RUSTFLAGS = "-Zthreads=4";

        buildInputs = with pkgs; [
          bacon
          cargo-expand
          cargo-llvm-cov
          cargo-nextest
          cargo-watch
          jq
          nixpkgs-fmt
          openssl
          pkg-config
          postgresql
          python3
          rust
          svix-cli
          sqlx-cli
        ];
      };
  };
}
