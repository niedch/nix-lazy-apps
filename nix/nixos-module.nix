let
  makeModule =
    self:
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      cfg = config.programs.nix-lazy-apps;
      system = pkgs.stdenv.hostPlatform.system or pkgs.system;

      # Resolve default package from flake or pkgs
      defaultLazyAppsPackage =
        if self != null && self ? packages.${system}.default then
          self.packages.${system}.default
        else if pkgs ? nix-lazy-apps then
          pkgs.nix-lazy-apps
        else
          throw "nix-lazy-apps: cannot find package. Please pass `self` from the flake to the module, add `nix-lazy-apps` to `pkgs` via an overlay, or set `programs.nix-lazy-apps.package`.";

      mkLazyApp = import ./mk-lazy-app.nix {
        inherit lib pkgs;
        lazyAppsPkg = cfg.package;
        follows = cfg.follows;
        defaultAllowUnfree = cfg.allowUnfree;
      };

      lazyAppPackages = map mkLazyApp cfg.lazyPkgs;
    in
    {
      options.programs.nix-lazy-apps = {
        enable = lib.mkOption {
          type = lib.types.bool;
          default = false;
          description = "Whether to enable nix-lazy-apps.";
        };

        lazyPkgs = lib.mkOption {
          type = lib.types.listOf lib.types.anything;
          default = [ ];
          description = "List of applications to generate lazy wrappers for (package derivations, flake strings, or attrsets).";
          example = lib.literalExpression "with pkgs; [ r2modman ]";
        };

        follows = lib.mkOption {
          type = lib.types.anything;
          default = null;
          description = "The nixpkgs flake input to follow (e.g. `inputs.nixpkgs`). Required when enabled unless all lazyPkgs specify full flake URIs.";
          example = lib.literalExpression "inputs.nixpkgs";
        };

        allowUnfree = lib.mkOption {
          type = lib.types.bool;
          default = config.nixpkgs.config.allowUnfree or false;
          defaultText = lib.literalExpression "config.nixpkgs.config.allowUnfree or false";
          description = "Whether to allow unfree packages in lazy wrappers by default.";
        };

        package = lib.mkOption {
          type = lib.types.package;
          default = defaultLazyAppsPackage;
          defaultText = lib.literalExpression "self.packages.\${pkgs.stdenv.hostPlatform.system or pkgs.system}.default or pkgs.nix-lazy-apps";
          description = "The nix-lazy-apps package to use.";
        };
      };

      config = lib.mkIf cfg.enable {
        environment.systemPackages = [ cfg.package ] ++ lazyAppPackages;
      };
    };
in
arg:
if arg ? config && arg ? lib then
  makeModule (arg.self or null) arg
else if arg ? self then
  makeModule arg.self
else
  makeModule arg
