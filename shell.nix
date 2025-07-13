{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    live-server
    cargo-watch
  ];

  shellHook = ''
    cargo-watch -x run &
    live-server --open=/output/index.html
  '';
}
