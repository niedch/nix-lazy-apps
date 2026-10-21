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

      # Resolve default package from flake or pkgs
      defaultLazyAppsPackage =
        if self != null && self ? packages.${pkgs.system}.default then
          self.packages.${pkgs.system}.default
        else if pkgs ? nix-lazy-apps then
          pkgs.nix-lazy-apps
        else
          throw "nix-lazy-apps: cannot find package. Please pass `self` from the flake to the module, add `nix-lazy-apps` to `pkgs` via an overlay, or set `programs.nix-lazy-apps.package`.";

      mkLazyApp = import ./mk-lazy-app.nix {
        inherit lib pkgs;
        lazyAppsPkg = cfg.package;
        defaultFlake = cfg.defaultFlake;
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

        defaultFlake = lib.mkOption {
          type = lib.types.str;
          default = "nixpkgs";
          description = "Default flake URI used when resolving lazy packages specified by name.";
        };

        package = lib.mkOption {
          type = lib.types.package;
          default = defaultLazyAppsPackage;
          defaultText = lib.literalExpression "self.packages.\${pkgs.system}.default or pkgs.nix-lazy-apps";
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
