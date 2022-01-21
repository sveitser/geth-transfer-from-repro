{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    # naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    # , naersk
    , rust-overlay
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        # pkgs = nixpkgs.legacyPackages."${system}";
        # naersk-lib = naersk.lib."${system}";
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      rec {

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [

            (rust-bin.stable."1.56.0".minimal.override {
              extensions = [ "rustfmt" "clippy" "llvm-tools-preview" "rust-src" ];
            })

            nodePackages.pnpm
            nodejs-16_x
            python3
            entr
            cargo-edit

          ] ++ lib.optionals stdenv.isDarwin [
            # required to compile ethers-rs
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.CoreFoundation

            # https://github.com/NixOS/nixpkgs/issues/126182
            libiconv
          ];

          shellHook = ''
            export PATH=$(pwd)/node_modules/.bin:$PATH
            export PATH=$(pwd)/bin:$PATH
          '';
        };
      }
    );
}
