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
    let
      topLevelOutputs = {
        nixosModules = rec {
          nix-lazy-apps = import ./nix/nixos-module.nix self;
          default = nix-lazy-apps;
        };

        homeManagerModules = rec {
          nix-lazy-apps = import ./nix/home-manager-module.nix self;
          default = nix-lazy-apps;
        };

        overlays = rec {
          nix-lazy-apps = final: prev: {
            nix-lazy-apps = self.packages.${final.stdenv.hostPlatform.system or final.system}.default;
          };
          default = nix-lazy-apps;
        };

        lib = {
          resolveFlake = import ./nix/resolve-flake.nix;
          mkLazyApp =
            args:
            let
              isPkgs = args ? stdenv && args ? lib;
              effectiveArgs = if isPkgs then { pkgs = args; } else args;
              pkgs = effectiveArgs.pkgs or (throw "lib.mkLazyApp: 'pkgs' argument is required");
              lazyAppsPkg =
                effectiveArgs.lazyAppsPkg or self.packages.${pkgs.stdenv.hostPlatform.system or pkgs.system}.default;
            in
            import ./nix/mk-lazy-app.nix ({ inherit lazyAppsPkg; } // effectiveArgs);
        };
      };
    in
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
      rec {
        packages = rec {
          nix-lazy-apps = pkgs.rustPlatform.buildRustPackage {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;

            # Tells Nix to use the local directory as the source code
            src = ./.;

            # Nix needs a hash of your Cargo.lock dependencies for reproducibility.
            cargoHash = "sha256-XR2zYsJhb7N2eR7zY3o0sHfaDr4njROHpOrKddnuMzU=";

            nativeBuildInputs = [ pkgs.makeWrapper ];

            postFixup = ''
              wrapProgram $out/bin/${cargoToml.package.name} \
                --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtimeLibs}
            '';
          };
          default = nix-lazy-apps;
        };

        legacyPackages = {
          mkLazyApp = import ./nix/mk-lazy-app.nix {
            inherit pkgs;
            lazyAppsPkg = packages.default;
            follows = nixpkgs;
          };
        };

        checks = {
          test-lazy-app-eval =
            let
              mkLazyApp = legacyPackages.mkLazyApp;
              mkLazyAppUnfreeDefault = import ./nix/mk-lazy-app.nix {
                inherit pkgs;
                lazyAppsPkg = packages.default;
                follows = nixpkgs;
                defaultAllowUnfree = true;
              };

              # Test cases
              # 1. pkgs.hello (free -> false)
              appHello = mkLazyApp pkgs.hello;

              # 2. pkgs.intellij-idea (unfree -> true)
              appIntellij = mkLazyApp pkgs.intellij-idea;

              # 3. Explicit unfree / impure attrsets
              appExplicitUnfree = mkLazyApp {
                package = pkgs.hello;
                unfree = true;
              };
              appExplicitFlakeUnfree = mkLazyApp {
                flakeOutput = "nixpkgs#hello";
                unfree = true;
              };
              appExplicitImpure = mkLazyApp {
                flakeOutput = "nixpkgs#hello";
                impure = true;
              };

              # 4. String default unfree
              appStringDefault = mkLazyApp "hello";
              appStringDefaultUnfree = mkLazyAppUnfreeDefault "hello";

              # 5. Terminal apps
              appBtopPackage = mkLazyApp {
                package = pkgs.btop;
                terminal = true;
              };
              appBtopFlake = mkLazyApp {
                flakeOutput = "nixpkgs#btop";
                name = "btop";
                terminal = true;
              };
              appDrvMetaTerminal = mkLazyApp (pkgs.hello // { meta = (pkgs.hello.meta or { }) // { terminal = true; }; });

              # Evaluation assertions
              asserts = [
                (pkgs.lib.hasPrefix "path:" appHello.passthru.spec.flakeOutput)
                (pkgs.lib.hasSuffix "#hello" appHello.passthru.spec.flakeOutput)
                (appHello.passthru.spec.follows == nixpkgs)
                (!appHello.passthru.spec.unfree)
                (!appHello.passthru.spec.impure)
                (!(pkgs.lib.hasInfix "--unfree" appHello.passthru.spec.exec))
                (!(pkgs.lib.hasInfix "--impure" appHello.passthru.spec.exec))
                (!appHello.passthru.spec.terminal)

                appIntellij.passthru.spec.unfree
                (pkgs.lib.hasInfix "--unfree" appIntellij.passthru.spec.exec)

                appExplicitUnfree.passthru.spec.unfree
                (pkgs.lib.hasInfix "--unfree" appExplicitUnfree.passthru.spec.exec)

                (appExplicitFlakeUnfree.passthru.spec.flakeOutput == "nixpkgs#hello")
                appExplicitFlakeUnfree.passthru.spec.unfree
                (pkgs.lib.hasInfix "--unfree" appExplicitFlakeUnfree.passthru.spec.exec)

                appExplicitImpure.passthru.spec.impure
                (pkgs.lib.hasInfix "--impure" appExplicitImpure.passthru.spec.exec)

                (!appStringDefault.passthru.spec.unfree)
                (!(pkgs.lib.hasInfix "--unfree" appStringDefault.passthru.spec.exec))

                appStringDefaultUnfree.passthru.spec.unfree
                (pkgs.lib.hasInfix "--unfree" appStringDefaultUnfree.passthru.spec.exec)

                appBtopPackage.passthru.spec.terminal
                appBtopFlake.passthru.spec.terminal
                appDrvMetaTerminal.passthru.spec.terminal
              ];

              evalSuccess = pkgs.lib.all (x: x) asserts;
            in
            if !evalSuccess then
              throw "test-lazy-app-eval assertions failed"
            else
              pkgs.runCommand "test-lazy-app-eval" { } ''
                # Verify generated wrapper scripts
                grep -F 'gui --flake-output' ${appHello}/bin/hello
                grep -F 'gui --unfree --flake-output' ${appIntellij}/bin/intellij-idea
                grep -F 'gui --unfree --flake-output' ${appExplicitUnfree}/bin/hello
                grep -F 'gui --unfree --flake-output' ${appExplicitFlakeUnfree}/bin/hello
                grep -F 'gui --impure --flake-output' ${appExplicitImpure}/bin/hello
                grep -F 'gui --flake-output' ${appStringDefault}/bin/hello
                grep -F 'gui --unfree --flake-output' ${appStringDefaultUnfree}/bin/hello

                # Verify desktop entries
                grep -F 'Terminal=false' ${appHello}/share/applications/hello.desktop
                grep -F 'Terminal=true' ${appBtopPackage}/share/applications/btop.desktop
                grep -F 'Terminal=true' ${appBtopFlake}/share/applications/btop.desktop
                grep -F 'Terminal=true' ${appDrvMetaTerminal}/share/applications/hello.desktop

                touch $out
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

          shellHook = ''
            mise tasks ls
          '';
        };
      }
    )
    // topLevelOutputs;
}
