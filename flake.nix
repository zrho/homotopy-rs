{
  description = "homotopy.io rust edition";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          nativeBuildInputs = with pkgs; [
            nodejs_latest
            (
              rust-bin.stable.latest.default.override {
                targets = [ "wasm32-unknown-unknown" ];
                extensions = [ "rust-src" ]; # seems to be already included in stable
              }
            )
            wasm-pack
          ];
        in
          {
            devShell = with pkgs; mkShell {
              buildInputs = nativeBuildInputs ++ [
                rust-analyzer
              ];
              RUST_SRC_PATH = "${rust-bin.stable.latest.rust-src}/lib/rustlib/src/rust/library";
            };
          }
    );
}
