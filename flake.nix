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
                effectiveArgs.lazyAppsPkg
                  or self.packages.${pkgs.stdenv.hostPlatform.system or pkgs.system}.default;
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
              appDrvMetaTerminal = mkLazyApp (
                pkgs.hello
                // {
                  meta = (pkgs.hello.meta or { }) // {
                    terminal = true;
                  };
                }
              );

              # 6. Desktop item metadata extraction
              mockDrvWithDesktopItems = pkgs.stdenv.mkDerivation {
                name = "mock-app";
                pname = "mock-app";
                desktopItems = [
                  (pkgs.makeDesktopItem {
                    name = "mock-app";
                    desktopName = "Mock Application";
                    genericName = "Mock Editor";
                    comment = "A mock application for testing";
                    icon = "mock-app-icon";
                    terminal = false;
                    categories = [
                      "Utility"
                      "TextEditor"
                    ];
                    mimeTypes = [
                      "text/plain"
                      "text/markdown"
                    ];
                    keywords = [
                      "mock"
                      "editor"
                    ];
                    startupWMClass = "mock-app-wm";
                  })
                ];
                buildCommand = "touch $out";
              };
              appMockWithDesktopItems = mkLazyApp mockDrvWithDesktopItems;

              # 7. Derivation with passthru.desktopItem and overrides
              mockDrvPassthru = pkgs.stdenv.mkDerivation {
                name = "mock-passthru";
                pname = "mock-passthru";
                passthru = {
                  desktopItem = pkgs.makeDesktopItem {
                    name = "mock-passthru";
                    desktopName = "Passthru App";
                    genericName = "Passthru Generic";
                    icon = "passthru-icon";
                    categories = [ "Audio" ];
                  };
                };
                buildCommand = "touch $out";
              };
              appMockPassthru = mkLazyApp {
                package = mockDrvPassthru;
                genericName = "Overridden Generic";
                categories = [ "AudioVideo" ];
              };

              # 8. Step 2 formatting and defaults test cases
              appObs = mkLazyApp "obs-studio";
              appVlcBin = mkLazyApp "vlc-bin";
              mockDrvGraphics =
                (pkgs.stdenv.mkDerivation {
                  name = "mock-graphics";
                  pname = "mock-graphics";
                  buildCommand = "touch $out";
                })
                // {
                  meta = {
                    position = "pkgs/applications/graphics/mock-graphics/default.nix:1";
                  };
                };
              appMockGraphics = mkLazyApp mockDrvGraphics;

              # 9. Step 3 custom exec test cases
              mockDrvWithExec = pkgs.stdenv.mkDerivation {
                name = "mock-exec-app";
                pname = "mock-exec-app";
                desktopItems = [
                  (pkgs.makeDesktopItem {
                    name = "mock-exec-app";
                    desktopName = "Mock Exec Application";
                    exec = "mock-exec-app -silent %U";
                  })
                ];
                buildCommand = "touch $out";
              };
              appMockWithExec = mkLazyApp mockDrvWithExec;

              appExplicitCustomExec = mkLazyApp {
                name = "custom-app";
                flakeOutput = "nixpkgs#custom-app";
                exec = "custom-app --open %f";
              };

              # 10. Step 4 icon handling and custom icon test cases
              mockCustomSvg = pkgs.writeText "custom-app.svg" ''<svg viewBox="0 0 100 100"><circle cx="50" cy="50" r="40"/></svg>'';
              appCustomSvg = mkLazyApp {
                name = "custom-svg-app";
                flakeOutput = "nixpkgs#custom-svg-app";
                icon = mockCustomSvg;
              };

              mockCustomPng = pkgs.runCommand "custom.png" { } "touch $out";
              appCustomPng = mkLazyApp {
                name = "custom-png-app";
                flakeOutput = "nixpkgs#custom-png-app";
                icon = mockCustomPng;
                iconName = "my-custom-icon";
              };

              mockIconDir = pkgs.runCommand "icon-dir" { } ''
                mkdir -p $out
                touch $out/dir-icon.png
                echo '<svg></svg>' > $out/dir-scalable.svg
              '';
              appCustomIconDir = mkLazyApp {
                name = "custom-dir-app";
                flakeOutput = "nixpkgs#custom-dir-app";
                icon = mockIconDir;
              };

              mockIconShareDir = pkgs.runCommand "icon-share-dir" { } ''
                mkdir -p $out/share/icons/hicolor/scalable/apps
                echo '<svg></svg>' > $out/share/icons/hicolor/scalable/apps/share-app.svg
              '';
              appCustomIconShareDir = mkLazyApp {
                name = "custom-share-app";
                flakeOutput = "nixpkgs#custom-share-app";
                icon = mockIconShareDir;
              };

              appCustomPath = mkLazyApp {
                name = "custom-path-app";
                flakeOutput = "nixpkgs#custom-path-app";
                icon = ./Cargo.toml;
                iconName = "path-icon";
              };

              appAbsolutePathIcon = mkLazyApp {
                name = "abs-path-app";
                flakeOutput = "nixpkgs#abs-path-app";
                icon = "/opt/myapp/share/icons/app.png";
              };

              appPlainStringIcon = mkLazyApp {
                name = "plain-string-app";
                flakeOutput = "nixpkgs#plain-string-app";
                icon = "custom-theme-icon";
              };

              appCustomSvgRelative = mkLazyApp {
                name = "custom-svg-relative-app";
                flakeOutput = "nixpkgs#custom-svg-relative-app";
                icon = mockCustomSvg;
                preferAbsoluteIcon = false;
              };

              appCustomPngRelative = mkLazyApp {
                name = "custom-png-relative-app";
                flakeOutput = "nixpkgs#custom-png-relative-app";
                icon = mockCustomPng;
                iconName = "relative-png-icon";
                preferAbsoluteIcon = false;
              };

              # 11. Step 5 real-world and edge case test cases
              appR2modman = mkLazyApp pkgs.r2modman;

              appEdgeCases = mkLazyApp {
                name = "edge-case-app";
                flakeOutput = "nixpkgs#edge-case-app";
                desktopName = "Edge Case Application";
                genericName = "Edge Case Generic";
                comment = "Edge case application test";
                icon = "edge-case-icon";
                categories = [
                  "Utility"
                  "Development"
                ];
                mimeTypes = [
                  "x-scheme-handler/test"
                  "text/plain"
                ];
                keywords = [
                  "edge"
                  "test case"
                ];
                startupWMClass = "EdgeCaseWM";
                exec = "edge-case-app --title \"Complex Title with Spaces\" --flag %U";
              };

              # Evaluation assertions
              asserts = [
                (pkgs.lib.hasPrefix "path:" appHello.passthru.spec.flakeOutput)
                (pkgs.lib.hasSuffix "#hello" appHello.passthru.spec.flakeOutput)
                (appHello.passthru.spec.follows == nixpkgs)
                (!appHello.passthru.spec.unfree)
                (!appHello.passthru.spec.impure)
                (!appHello.passthru.spec.terminal)

                appIntellij.passthru.spec.unfree
                appExplicitUnfree.passthru.spec.unfree
                (appExplicitFlakeUnfree.passthru.spec.flakeOutput == "nixpkgs#hello")
                appExplicitFlakeUnfree.passthru.spec.unfree
                appExplicitImpure.passthru.spec.impure
                (!appStringDefault.passthru.spec.unfree)
                appStringDefaultUnfree.passthru.spec.unfree

                appBtopPackage.passthru.spec.terminal
                appBtopFlake.passthru.spec.terminal
                appDrvMetaTerminal.passthru.spec.terminal

                (appMockWithDesktopItems.passthru.spec.desktopName == "Mock Application")
                (appMockWithDesktopItems.passthru.spec.genericName == "Mock Editor")
                (appMockWithDesktopItems.passthru.spec.comment == "A mock application for testing")
                (appMockWithDesktopItems.passthru.spec.icon == "mock-app-icon")
                (!appMockWithDesktopItems.passthru.spec.terminal)
                (
                  appMockWithDesktopItems.passthru.spec.categories == [
                    "Utility"
                    "TextEditor"
                  ]
                )
                (
                  appMockWithDesktopItems.passthru.spec.mimeTypes == [
                    "text/plain"
                    "text/markdown"
                  ]
                )
                (
                  appMockWithDesktopItems.passthru.spec.keywords == [
                    "mock"
                    "editor"
                  ]
                )
                (appMockWithDesktopItems.passthru.spec.startupWMClass == "mock-app-wm")

                (appMockPassthru.passthru.spec.desktopName == "Passthru App")
                (appMockPassthru.passthru.spec.genericName == "Overridden Generic")
                (appMockPassthru.passthru.spec.categories == [ "AudioVideo" ])
                (appMockPassthru.passthru.spec.icon == "passthru-icon")

                # Step 2 assertions: desktopName formatting
                (appHello.passthru.spec.desktopName == "Hello")
                (appObs.passthru.spec.desktopName == "OBS Studio")
                (appVlcBin.passthru.spec.desktopName == "VLC")

                # Step 2 assertions: category defaults and inference
                (appHello.passthru.spec.categories == [ "Utility" ])
                (
                  appBtopPackage.passthru.spec.categories == [
                    "System"
                    "TerminalEmulator"
                  ]
                )
                (appMockGraphics.passthru.spec.categories == [ "Graphics" ])

                # Step 2 assertions: default keywords generation
                (appHello.passthru.spec.keywords == [ "hello" ])
                (
                  appObs.passthru.spec.keywords == [
                    "obs-studio"
                    "obs"
                    "studio"
                  ]
                )
                (
                  appBtopPackage.passthru.spec.keywords == [
                    "btop"
                    "terminal"
                    "cli"
                  ]
                )
                (
                  appVlcBin.passthru.spec.keywords == [
                    "vlc-bin"
                    "vlc"
                    "bin"
                  ]
                )

                # Step 3 assertions: Exec routing through wrapperScript and arg passing
                (appHello.passthru.spec.exec == "${appHello.passthru.wrapperScript}/bin/hello %U")
                (appIntellij.passthru.spec.exec == "${appIntellij.passthru.wrapperScript}/bin/intellij-idea %U")
                (appBtopPackage.passthru.spec.exec == "${appBtopPackage.passthru.wrapperScript}/bin/btop")
                (
                  appMockWithExec.passthru.spec.exec
                  == "${appMockWithExec.passthru.wrapperScript}/bin/mock-exec-app -silent %U"
                )
                (
                  appExplicitCustomExec.passthru.spec.exec
                  == "${appExplicitCustomExec.passthru.wrapperScript}/bin/custom-app --open %f"
                )

                # Step 4 assertions: icon handling
                # Default icons for non-terminal and terminal apps
                (appHello.passthru.spec.icon == "hello")
                (appBtopPackage.passthru.spec.icon == "utilities-terminal")
                (appBtopFlake.passthru.spec.icon == "utilities-terminal")
                (appDrvMetaTerminal.passthru.spec.icon == "utilities-terminal")

                # Custom icon derivation / path specs
                (
                  appCustomSvg.passthru.spec.icon
                  == "${appCustomSvg.passthru.customIconPkg}/share/icons/hicolor/scalable/apps/custom-svg-app.svg"
                )
                (appCustomSvg.passthru ? customIconPkg)
                (
                  appCustomPng.passthru.spec.icon
                  == "${appCustomPng.passthru.customIconPkg}/share/icons/hicolor/256x256/apps/my-custom-icon.png"
                )
                (appCustomPng.passthru ? customIconPkg)
                (appCustomIconDir.passthru.spec.icon == "custom-dir-app")
                (appCustomIconDir.passthru ? customIconPkg)
                (appCustomIconShareDir.passthru.spec.icon == "custom-share-app")
                (appCustomIconShareDir.passthru ? customIconPkg)
                (appCustomPath.passthru.spec.icon == "path-icon")
                (appCustomPath.passthru ? customIconPkg)
                (appCustomSvgRelative.passthru.spec.icon == "custom-svg-relative-app")
                (appCustomPngRelative.passthru.spec.icon == "relative-png-icon")

                # Absolute path and plain string icons (no customIconPkg)
                (appAbsolutePathIcon.passthru.spec.icon == "/opt/myapp/share/icons/app.png")
                (!(appAbsolutePathIcon.passthru ? customIconPkg))
                (appPlainStringIcon.passthru.spec.icon == "custom-theme-icon")
                (!(appPlainStringIcon.passthru ? customIconPkg))

                # Step 5 assertions: real-world r2modman and edge-case-app
                (appR2modman.passthru.spec.name == "r2modman")
                (appR2modman.passthru.spec.desktopName == "r2modman")
                (appR2modman.passthru.spec.comment == "Unofficial Thunderstore mod manager")
                (appR2modman.passthru.spec.icon == "r2modman")
                (appR2modman.passthru.spec.categories == [ "Game" ])
                (appR2modman.passthru.spec.mimeTypes == [ "x-scheme-handler/ror2mm" ])
                (
                  appR2modman.passthru.spec.keywords == [
                    "launcher"
                    "mod manager"
                    "thunderstore"
                  ]
                )
                (!appR2modman.passthru.spec.terminal)
                (appR2modman.passthru.spec.exec == "${appR2modman.passthru.wrapperScript}/bin/r2modman %U")

                (appEdgeCases.passthru.spec.name == "edge-case-app")
                (appEdgeCases.passthru.spec.desktopName == "Edge Case Application")
                (appEdgeCases.passthru.spec.genericName == "Edge Case Generic")
                (appEdgeCases.passthru.spec.comment == "Edge case application test")
                (appEdgeCases.passthru.spec.icon == "edge-case-icon")
                (
                  appEdgeCases.passthru.spec.categories == [
                    "Utility"
                    "Development"
                  ]
                )
                (
                  appEdgeCases.passthru.spec.mimeTypes == [
                    "x-scheme-handler/test"
                    "text/plain"
                  ]
                )
                (
                  appEdgeCases.passthru.spec.keywords == [
                    "edge"
                    "test case"
                  ]
                )
                (appEdgeCases.passthru.spec.startupWMClass == "EdgeCaseWM")
                (
                  appEdgeCases.passthru.spec.exec
                  == "${appEdgeCases.passthru.wrapperScript}/bin/edge-case-app --title \"Complex Title with Spaces\" --flag %U"
                )
              ];

              evalSuccess = pkgs.lib.all (x: x) asserts;
            in
            if !evalSuccess then
              throw "test-lazy-app-eval assertions failed"
            else
              pkgs.runCommand "test-lazy-app-eval" {
                nativeBuildInputs = [ pkgs.desktop-file-utils ];
              } ''
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

                # Verify extracted desktop items in .desktop files
                grep -F 'Name=Mock Application' ${appMockWithDesktopItems}/share/applications/mock-app.desktop
                grep -F 'GenericName=Mock Editor' ${appMockWithDesktopItems}/share/applications/mock-app.desktop
                grep -F 'Comment=A mock application for testing' ${appMockWithDesktopItems}/share/applications/mock-app.desktop
                grep -F 'Icon=mock-app-icon' ${appMockWithDesktopItems}/share/applications/mock-app.desktop
                grep -F 'Categories=Utility;TextEditor' ${appMockWithDesktopItems}/share/applications/mock-app.desktop
                grep -F 'MimeType=text/plain;text/markdown' ${appMockWithDesktopItems}/share/applications/mock-app.desktop
                grep -F 'Keywords=mock;editor' ${appMockWithDesktopItems}/share/applications/mock-app.desktop
                grep -F 'StartupWMClass=mock-app-wm' ${appMockWithDesktopItems}/share/applications/mock-app.desktop

                grep -F 'GenericName=Overridden Generic' ${appMockPassthru}/share/applications/mock-passthru.desktop
                grep -F 'Categories=AudioVideo' ${appMockPassthru}/share/applications/mock-passthru.desktop

                # Verify Step 2 desktop file formatting and defaults
                grep -F 'Name=Hello' ${appHello}/share/applications/hello.desktop
                grep -F 'Categories=Utility' ${appHello}/share/applications/hello.desktop
                grep -F 'Keywords=hello' ${appHello}/share/applications/hello.desktop

                grep -F 'Name=OBS Studio' ${appObs}/share/applications/obs-studio.desktop
                grep -F 'Categories=Utility' ${appObs}/share/applications/obs-studio.desktop
                grep -F 'Keywords=obs-studio;obs;studio' ${appObs}/share/applications/obs-studio.desktop

                grep -F 'Name=VLC' ${appVlcBin}/share/applications/vlc-bin.desktop
                grep -F 'Keywords=vlc-bin;vlc;bin' ${appVlcBin}/share/applications/vlc-bin.desktop

                grep -F 'Categories=System;TerminalEmulator' ${appBtopPackage}/share/applications/btop.desktop
                grep -F 'Keywords=btop;terminal;cli' ${appBtopPackage}/share/applications/btop.desktop

                grep -F 'Categories=Graphics' ${appMockGraphics}/share/applications/mock-graphics.desktop

                # Verify Step 3: wrapper scripts include -- "$@"
                grep -F -- '-- "$@"' ${appHello}/bin/hello
                grep -F -- '-- "$@"' ${appIntellij}/bin/intellij-idea

                # Verify Step 3: desktop entries Exec= route through wrapperScript with appropriate args
                grep -F "Exec=${appHello.passthru.wrapperScript}/bin/hello %U" ${appHello}/share/applications/hello.desktop
                grep -F "Exec=${appBtopPackage.passthru.wrapperScript}/bin/btop" ${appBtopPackage}/share/applications/btop.desktop
                grep -F "Exec=${appMockWithExec.passthru.wrapperScript}/bin/mock-exec-app -silent %U" ${appMockWithExec}/share/applications/mock-exec-app.desktop
                grep -F "Exec=${appExplicitCustomExec.passthru.wrapperScript}/bin/custom-app --open %f" ${appExplicitCustomExec}/share/applications/custom-app.desktop

                # Verify Step 4: terminal app fallback to utilities-terminal
                grep -F 'Icon=utilities-terminal' ${appBtopPackage}/share/applications/btop.desktop
                grep -F 'Icon=utilities-terminal' ${appBtopFlake}/share/applications/btop.desktop
                grep -F 'Icon=utilities-terminal' ${appDrvMetaTerminal}/share/applications/hello.desktop

                # Verify Step 4: custom SVG icon installation to pixmaps and hicolor scalable apps
                test -f ${appCustomSvg}/share/pixmaps/custom-svg-app.svg
                test -f ${appCustomSvg}/share/icons/hicolor/scalable/apps/custom-svg-app.svg
                grep -F "Icon=${appCustomSvg.passthru.customIconPkg}/share/icons/hicolor/scalable/apps/custom-svg-app.svg" ${appCustomSvg}/share/applications/custom-svg-app.desktop
                grep -F 'Icon=custom-svg-relative-app' ${appCustomSvgRelative}/share/applications/custom-svg-relative-app.desktop

                # Verify Step 4: custom PNG icon with iconName installation to pixmaps and standard hicolor sizes
                test -f ${appCustomPng}/share/pixmaps/my-custom-icon.png
                test -f ${appCustomPng}/share/icons/hicolor/256x256/apps/my-custom-icon.png
                test -f ${appCustomPng}/share/icons/hicolor/128x128/apps/my-custom-icon.png
                test -f ${appCustomPng}/share/icons/hicolor/64x64/apps/my-custom-icon.png
                test -f ${appCustomPng}/share/icons/hicolor/48x48/apps/my-custom-icon.png
                grep -F "Icon=${appCustomPng.passthru.customIconPkg}/share/icons/hicolor/256x256/apps/my-custom-icon.png" ${appCustomPng}/share/applications/custom-png-app.desktop
                grep -F 'Icon=relative-png-icon' ${appCustomPngRelative}/share/applications/custom-png-relative-app.desktop

                # Verify Step 4: directory without share installs into pixmaps, scalable apps, and standard png sizes
                test -f ${appCustomIconDir}/share/pixmaps/dir-icon.png
                test -f ${appCustomIconDir}/share/pixmaps/dir-scalable.svg
                test -f ${appCustomIconDir}/share/icons/hicolor/scalable/apps/dir-scalable.svg
                test -f ${appCustomIconDir}/share/icons/hicolor/256x256/apps/dir-icon.png
                test -f ${appCustomIconDir}/share/icons/hicolor/128x128/apps/dir-icon.png
                test -f ${appCustomIconDir}/share/icons/hicolor/64x64/apps/dir-icon.png
                test -f ${appCustomIconDir}/share/icons/hicolor/48x48/apps/dir-icon.png
                grep -F 'Icon=custom-dir-app' ${appCustomIconDir}/share/applications/custom-dir-app.desktop

                # Verify Step 4: directory with share installs into share
                test -f ${appCustomIconShareDir}/share/icons/hicolor/scalable/apps/share-app.svg
                grep -F 'Icon=custom-share-app' ${appCustomIconShareDir}/share/applications/custom-share-app.desktop

                # Verify Step 4: custom path (builtins.isPath) installs into pixmaps
                test -f ${appCustomPath}/share/pixmaps/path-icon.toml
                grep -F 'Icon=path-icon' ${appCustomPath}/share/applications/custom-path-app.desktop

                # Verify Step 4: absolute path and plain string desktop entry Icon=
                grep -F 'Icon=/opt/myapp/share/icons/app.png' ${appAbsolutePathIcon}/share/applications/abs-path-app.desktop
                grep -F 'Icon=custom-theme-icon' ${appPlainStringIcon}/share/applications/plain-string-app.desktop

                # Step 5: Validate all generated .desktop files
                desktop-file-validate \
                  ${appHello}/share/applications/hello.desktop \
                  ${appIntellij}/share/applications/intellij-idea.desktop \
                  ${appExplicitUnfree}/share/applications/hello.desktop \
                  ${appExplicitFlakeUnfree}/share/applications/hello.desktop \
                  ${appExplicitImpure}/share/applications/hello.desktop \
                  ${appStringDefault}/share/applications/hello.desktop \
                  ${appStringDefaultUnfree}/share/applications/hello.desktop \
                  ${appBtopPackage}/share/applications/btop.desktop \
                  ${appBtopFlake}/share/applications/btop.desktop \
                  ${appDrvMetaTerminal}/share/applications/hello.desktop \
                  ${appMockWithDesktopItems}/share/applications/mock-app.desktop \
                  ${appMockPassthru}/share/applications/mock-passthru.desktop \
                  ${appObs}/share/applications/obs-studio.desktop \
                  ${appVlcBin}/share/applications/vlc-bin.desktop \
                  ${appMockGraphics}/share/applications/mock-graphics.desktop \
                  ${appMockWithExec}/share/applications/mock-exec-app.desktop \
                  ${appExplicitCustomExec}/share/applications/custom-app.desktop \
                  ${appCustomSvg}/share/applications/custom-svg-app.desktop \
                  ${appCustomSvgRelative}/share/applications/custom-svg-relative-app.desktop \
                  ${appCustomPng}/share/applications/custom-png-app.desktop \
                  ${appCustomPngRelative}/share/applications/custom-png-relative-app.desktop \
                  ${appCustomIconDir}/share/applications/custom-dir-app.desktop \
                  ${appCustomIconShareDir}/share/applications/custom-share-app.desktop \
                  ${appCustomPath}/share/applications/custom-path-app.desktop \
                  ${appAbsolutePathIcon}/share/applications/abs-path-app.desktop \
                  ${appPlainStringIcon}/share/applications/plain-string-app.desktop \
                  ${appR2modman}/share/applications/r2modman.desktop \
                  ${appEdgeCases}/share/applications/edge-case-app.desktop

                # Verify Step 5: r2modman upstream desktopItem extraction
                grep -F 'Name=r2modman' ${appR2modman}/share/applications/r2modman.desktop
                grep -F 'Comment=Unofficial Thunderstore mod manager' ${appR2modman}/share/applications/r2modman.desktop
                grep -F 'Icon=r2modman' ${appR2modman}/share/applications/r2modman.desktop
                grep -F 'Categories=Game' ${appR2modman}/share/applications/r2modman.desktop
                grep -F 'MimeType=x-scheme-handler/ror2mm' ${appR2modman}/share/applications/r2modman.desktop
                grep -F 'Keywords=launcher;mod manager;thunderstore' ${appR2modman}/share/applications/r2modman.desktop
                grep -F "Exec=${appR2modman.passthru.wrapperScript}/bin/r2modman %U" ${appR2modman}/share/applications/r2modman.desktop

                # Verify Step 5: edge case app with quoted args, mimeTypes, keywords, genericName, startupWMClass
                grep -F 'Name=Edge Case Application' ${appEdgeCases}/share/applications/edge-case-app.desktop
                grep -F 'GenericName=Edge Case Generic' ${appEdgeCases}/share/applications/edge-case-app.desktop
                grep -F 'Comment=Edge case application test' ${appEdgeCases}/share/applications/edge-case-app.desktop
                grep -F 'Icon=edge-case-icon' ${appEdgeCases}/share/applications/edge-case-app.desktop
                grep -F 'Categories=Utility;Development' ${appEdgeCases}/share/applications/edge-case-app.desktop
                grep -F 'MimeType=x-scheme-handler/test;text/plain' ${appEdgeCases}/share/applications/edge-case-app.desktop
                grep -F 'Keywords=edge;test case' ${appEdgeCases}/share/applications/edge-case-app.desktop
                grep -F 'StartupWMClass=EdgeCaseWM' ${appEdgeCases}/share/applications/edge-case-app.desktop
                grep -F "Exec=${appEdgeCases.passthru.wrapperScript}/bin/edge-case-app --title \"Complex Title with Spaces\" --flag %U" ${appEdgeCases}/share/applications/edge-case-app.desktop

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
