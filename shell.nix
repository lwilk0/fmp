{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    glib
    gobject-introspection  # Often needed for GTK bindings
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
