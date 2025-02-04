{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShellNoCC {
  packages = with pkgs; [
    cargo
    cargo-generate
  ];
  shellHook = ''
    nvim
  '';
}
