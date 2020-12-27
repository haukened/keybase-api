{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkgconfig cacert
  ];
  buildInputs = with pkgs; [
    # These deps should suffice for now
    binutils gcc gnumake openssl
  ];
}
