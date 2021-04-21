{ pkgs ? import <nixpkgs> {} }:
let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  moz_nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
with pkgs;
mkShell {
  buildInputs = [
    nodejs_latest
    (moz_nixpkgs.latest.rustChannels.stable.rust.override {
      targets = ["wasm32-unknown-unknown"];
      extensions = ["rust-src"];
    })
    gcc wabt wasm-bindgen-cli wasm-pack
  ];
}
