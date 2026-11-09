{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    clang                       
    llvmPackages.libclang.lib
    pkg-config

    rustc
    cargo
    rust-analyzer
    rustfmt
    clippy
  ];

  buildInputs = with pkgs; [
    nettle
    gmp
  ];

  shellHook = ''
    export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
    export PKG_CONFIG_PATH="${pkgs.nettle.dev}/lib/pkgconfig:${pkgs.gmp.dev}/lib/pkgconfig"
    export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}"
  '';
}