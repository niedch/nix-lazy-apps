{
  description = "Lazy nix apps flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Read the Cargo.toml to get the package name
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;

          # Tells Nix to use the local directory as the source code
          src = ./.;

          # Nix needs a hash of your Cargo.lock dependencies for reproducibility.
          cargoHash = "sha256-nPLfGpAaCCG/q1+LhZ/EBKT3QnA18ZFHjiu1pFvIb04=";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            mise
          ];

          # Optional: sets up environment variables like RUST_SRC_PATH
          shellHook = ''
            mise tasks ls
          '';
        };
      }
    );
}
