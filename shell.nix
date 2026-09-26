{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    pkg-config
    glib
    gobject-introspection
    gtk4
  ];

  buildInputs = with pkgs; [
    gtk4
    libgpg-error
    cairo
    pango
    libadwaita
    gpgme
  ];
}
