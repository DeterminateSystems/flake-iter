# This flake was initially generated by fh, the CLI for FlakeHub (version 0.1.10)
{
  description = "flake-iter";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2405.*";
    fenix = { url = "https://flakehub.com/f/nix-community/fenix/0.1.1885"; inputs.nixpkgs.follows = "nixpkgs"; };
    crane = { url = "https://flakehub.com/f/ipetkov/crane/0.17.3"; inputs.nixpkgs.follows = "nixpkgs"; };
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/*";
    flake-schemas.url = "https://flakehub.com/f/DeterminateSystems/flake-schemas/*";
  };

  outputs = { self, nixpkgs, fenix, crane, flake-compat, flake-schemas }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-darwin" "x86_64-darwin" "aarch64-linux" ];
      forEachSupportedSystem = f: nixpkgs.lib.genAttrs supportedSystems (system: f {
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
      });
      meta = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
    in
    {
      inherit (flake-schemas) schemas;

      overlays.default = final: prev: rec {
        system = final.stdenv.hostPlatform.system;
        rustToolchain = with fenix.packages.${system};
          combine ([
            stable.clippy
            stable.rustc
            stable.cargo
            stable.rustfmt
            stable.rust-src
          ] ++ nixpkgs.lib.optionals (system == "x86_64-linux") [
            targets.x86_64-unknown-linux-musl.stable.rust-std
          ] ++ nixpkgs.lib.optionals (system == "aarch64-linux") [
            targets.aarch64-unknown-linux-musl.stable.rust-std
          ]);
        craneLib = (crane.mkLib final).overrideToolchain rustToolchain;
      };

      devShells = forEachSupportedSystem ({ pkgs }: rec {
        default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-edit
            bacon
            rust-analyzer
            nixpkgs-fmt
            cargo-machete
            iconv
          ];

          env = {
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };
        };

        a = default;
        b = default;
        c = default;
        d = default;
      });

      # These outputs are solely for local testing
      packages = forEachSupportedSystem ({ pkgs }: rec {
        default = pkgs.craneLib.buildPackage {
          pname = meta.name;
          inherit (meta) version;
          src = self;
          doCheck = true;
          buildInputs = with pkgs; [ iconv ];
        };

        a = default;
        b = default;
        c = default;
        d = pkgs.jq;
        e = pkgs.ponysay;
        f = pkgs.hello;
      });
    };
}
