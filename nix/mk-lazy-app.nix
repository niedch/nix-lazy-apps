{
  lib ? pkgs.lib,
  pkgs,
  lazyAppsPkg,
  defaultFlake ? "nixpkgs",
  ...
}@args:

let
  # Strip string context to avoid turning lazy targets into build dependencies
  safeString =
    s:
    if s != null && builtins.isString s then
      builtins.unsafeDiscardStringContext s
    else
      s;

  # Clean description for desktop files (must not contain newlines)
  sanitizeComment =
    s:
    let
      clean = safeString s;
    in
    if clean == null then
      null
    else
      builtins.head (lib.splitString "\n" clean);

  # Parse plain string: "pkg", "flake#pkg", or "github:org/repo#pkg"
  parseString =
    str:
    let
      cleanStr = safeString str;
    in
    if lib.hasInfix "#" cleanStr then
      let
        parts = lib.splitString "#" cleanStr;
        attrPart = builtins.elemAt parts (builtins.length parts - 1);
        attrSubParts = lib.splitString "." attrPart;
        lastAttr = lib.last attrSubParts;
      in
      {
        name = lastAttr;
        flakeOutput = cleanStr;
      }
    else
      {
        name = cleanStr;
        flakeOutput = "${defaultFlake}#${cleanStr}";
      };

  # Parse derivation attributes safely without realizing the derivation
  parseDerivation =
    drv:
    let
      rawPname =
        drv.pname or (
          let
            parsed = builtins.parseDrvName (drv.name or "");
          in
          if parsed.name != "" then parsed.name else drv.name or "unknown-app"
        );
      name = safeString rawPname;
      desktopName = safeString (drv.meta.desktopName or drv.meta.mainProgram or name);
      comment = sanitizeComment (drv.meta.description or null);
    in
    {
      inherit name desktopName comment;
      flakeOutput = "${defaultFlake}#${name}";
      icon = name;
    };

  # Main builder implementation
  mkLazyApp =
    arg:
    let
      rawSpec =
        if builtins.isString arg then
          parseString arg
        else if lib.isDerivation arg then
          parseDerivation arg
        else if builtins.isAttrs arg then
          let
            base =
              if arg ? package && lib.isDerivation arg.package then
                parseDerivation arg.package
              else if arg ? flakeOutput && !(arg ? name) then
                parseString arg.flakeOutput
              else
                { };
          in
          base // arg
        else
          throw "mkLazyApp: argument must be a derivation, string, or attribute set, but got ${builtins.typeOf arg}";

      name = safeString (
        rawSpec.name or (throw "mkLazyApp: unable to determine 'name' for application")
      );
      flakeOutput = safeString (rawSpec.flakeOutput or "${defaultFlake}#${name}");
      desktopName = safeString (rawSpec.desktopName or name);
      icon = if rawSpec ? icon then safeString rawSpec.icon else name;
      comment = sanitizeComment (rawSpec.comment or null);
      categories = rawSpec.categories or [ ];
      terminal = rawSpec.terminal or false;
      exec =
        rawSpec.exec or "${lazyAppsPkg}/bin/nix-lazy-apps gui --flake-output \"${flakeOutput}\" --name \"${name}\"";

      spec = {
        inherit
          name
          flakeOutput
          desktopName
          icon
          comment
          categories
          terminal
          exec
          ;
      };

      desktopItem = pkgs.makeDesktopItem (
        {
          inherit (spec)
            name
            desktopName
            exec
            categories
            terminal
            ;
        }
        // lib.optionalAttrs (spec.icon != null) { inherit (spec) icon; }
        // lib.optionalAttrs (spec.comment != null) { inherit (spec) comment; }
        // (rawSpec.extraDesktopConfig or { })
      );

      wrapperScript = pkgs.writeShellScriptBin spec.name ''
        exec ${lazyAppsPkg}/bin/nix-lazy-apps gui --flake-output ${lib.escapeShellArg spec.flakeOutput} --name ${lib.escapeShellArg spec.name} "$@"
      '';
    in
    pkgs.symlinkJoin {
      inherit (spec) name;
      paths = [
        desktopItem
        wrapperScript
      ] ++ (rawSpec.extraPaths or [ ]);

      meta = (rawSpec.meta or { }) // {
        mainProgram = spec.name;
        description = spec.comment;
      };

      passthru = {
        inherit desktopItem wrapperScript spec;
      } // (rawSpec.passthru or { });
    };
in
if args ? app then
  mkLazyApp args.app
else if args ? package then
  mkLazyApp args.package
else
  mkLazyApp
