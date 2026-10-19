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

        runtimeLibs = with pkgs; [
          # Wayland
          wayland
          libxkbcommon

          # Graphics / Vulkan / OpenGL
          libGL
          vulkan-loader

          # X11 fallback
          libx11
          libxcursor
          libxi
          libxrandr
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;

          # Tells Nix to use the local directory as the source code
          src = ./.;

          # Nix needs a hash of your Cargo.lock dependencies for reproducibility.
          cargoHash = "sha256-koH4YcY/tKffDrXeHCBH2gFk1dfupOt9c/tN5OxWQX4=";

          nativeBuildInputs = [ pkgs.makeWrapper ];

          postFixup = ''
            wrapProgram $out/bin/${cargoToml.package.name} \
              --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtimeLibs}
          '';
        };


        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs =
            (with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
              rust-analyzer
              mise
            ])
            ++ runtimeLibs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;

          # Optional: sets up environment variables like RUST_SRC_PATH
          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath runtimeLibs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
            mise tasks ls
          '';
        };
      }
    );
}
