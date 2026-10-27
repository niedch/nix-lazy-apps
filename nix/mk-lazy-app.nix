{
  lib ? pkgs.lib,
  pkgs,
  lazyAppsPkg,
  follows ? null,
  defaultAllowUnfree ? false,
  ...
}@args:

let
  resolveFlake = import ./resolve-flake.nix { inherit lib; };

  # Strip string context to avoid turning lazy targets into build dependencies
  safeString =
    s: if s != null && builtins.isString s then builtins.unsafeDiscardStringContext s else s;

  targetFlake = if follows != null then resolveFlake follows else null;

  # Clean description for desktop files (must not contain newlines)
  sanitizeComment =
    s:
    let
      clean = safeString s;
    in
    if clean == null then null else builtins.head (lib.splitString "\n" clean);

  # Capitalize first letter of string
  capitalize =
    s:
    if s == null || s == "" then
      ""
    else
      (lib.toUpper (builtins.substring 0 1 s)) + (builtins.substring 1 (-1) s);

  knownAcronyms = {
    "obs" = "OBS";
    "vlc" = "VLC";
    "gimp" = "GIMP";
    "2d" = "2D";
    "3d" = "3D";
    "ide" = "IDE";
    "cli" = "CLI";
    "gui" = "GUI";
    "ui" = "UI";
    "os" = "OS";
    "pdf" = "PDF";
    "sql" = "SQL";
    "url" = "URL";
    "uri" = "URI";
    "ssh" = "SSH";
    "ftp" = "FTP";
    "sftp" = "SFTP";
    "http" = "HTTP";
    "https" = "HTTPS";
    "ip" = "IP";
    "vpn" = "VPN";
    "sdk" = "SDK";
    "api" = "API";
    "cpu" = "CPU";
    "gpu" = "GPU";
    "ram" = "RAM";
    "rom" = "ROM";
    "io" = "IO";
    "ai" = "AI";
    "db" = "DB";
    "vm" = "VM";
    "cad" = "CAD";
    "cam" = "CAM";
    "xml" = "XML";
    "json" = "JSON";
    "yaml" = "YAML";
    "toml" = "TOML";
    "html" = "HTML";
    "css" = "CSS";
    "av" = "AV";
    "dj" = "DJ";
    "tv" = "TV";
    "cd" = "CD";
    "dvd" = "DVD";
    "vr" = "VR";
    "ar" = "AR";
    "xr" = "XR";
    "lan" = "LAN";
    "wan" = "WAN";
    "wifi" = "WiFi";
    "wi-fi" = "Wi-Fi";
    "bluetooth" = "Bluetooth";
    "usb" = "USB";
    "pci" = "PCI";
    "pcie" = "PCIe";
    "nvme" = "NVMe";
    "ssd" = "SSD";
    "hdd" = "HDD";
  };

  # Strip packaging suffixes like -bin, -wrapped, -unwrapped
  stripPackagingSuffixes =
    str:
    let
      clean = safeString str;
    in
    if clean == null then
      null
    else
      let
        suffixes = [
          "-unwrapped"
          "_unwrapped"
          "-wrapped"
          "_wrapped"
          "-bin"
          "_bin"
        ];
        stripOne =
          s:
          let
            matched = lib.filter (suf: lib.hasSuffix suf s && s != suf) suffixes;
          in
          if matched != [ ] then lib.removeSuffix (builtins.head matched) s else s;
        s1 = stripOne clean;
        s2 = stripOne s1;
      in
      s2;

  # Format kebab-case / snake_case into human-readable title-cased name, preserving acronyms
  formatDesktopName =
    name:
    let
      clean = safeString name;
    in
    if clean == null || clean == "" then
      clean
    else
      let
        stripped = stripPackagingSuffixes clean;
        normalized = builtins.replaceStrings [ "_" " " ] [ "-" "-" ] stripped;
        parts = builtins.filter (x: x != "") (lib.splitString "-" normalized);
        formatToken =
          token:
          let
            lower = lib.toLower token;
          in
          if knownAcronyms ? ${lower} then knownAcronyms.${lower} else capitalize token;
        formattedParts = builtins.map formatToken parts;
      in
      if formattedParts == [ ] then clean else lib.concatStringsSep " " formattedParts;

  # Infer freedesktop registered category from derivation position path
  inferCategoriesFromPosition =
    pos:
    let
      cleanPos = safeString pos;
    in
    if cleanPos == null || cleanPos == "" then
      [ ]
    else if lib.hasInfix "applications/graphics" cleanPos || lib.hasInfix "/graphics/" cleanPos then
      [ "Graphics" ]
    else if lib.hasInfix "applications/office" cleanPos || lib.hasInfix "/office/" cleanPos then
      [ "Office" ]
    else if lib.hasInfix "applications/audio" cleanPos || lib.hasInfix "/audio/" cleanPos then
      [
        "AudioVideo"
        "Audio"
      ]
    else if lib.hasInfix "applications/video" cleanPos || lib.hasInfix "/video/" cleanPos then
      [
        "AudioVideo"
        "Video"
      ]
    else if lib.hasInfix "applications/networking" cleanPos || lib.hasInfix "/networking/" cleanPos then
      [ "Network" ]
    else if lib.hasInfix "applications/editors" cleanPos || lib.hasInfix "/editors/" cleanPos then
      [
        "Utility"
        "TextEditor"
      ]
    else if lib.hasInfix "applications/science" cleanPos || lib.hasInfix "/science/" cleanPos then
      [ "Science" ]
    else if lib.hasInfix "applications/version-management" cleanPos then
      [ "Development" ]
    else if lib.hasInfix "development/" cleanPos || lib.hasInfix "/development" cleanPos then
      [ "Development" ]
    else if lib.hasInfix "games/" cleanPos || lib.hasInfix "/games" cleanPos then
      [ "Game" ]
    else if lib.hasInfix "tools/package-management" cleanPos then
      [
        "System"
        "PackageManager"
      ]
    else if lib.hasInfix "tools/security" cleanPos then
      [
        "System"
        "Security"
      ]
    else if lib.hasInfix "tools/system" cleanPos then
      [ "System" ]
    else
      [ ];

  # Generate search keywords from name, mainProgram, terminal
  buildDefaultKeywords =
    {
      name,
      mainProgram ? null,
      terminal ? false,
    }:
    let
      cleanName = safeString name;
      lowerName = lib.toLower cleanName;
      stripped = stripPackagingSuffixes lowerName;
      tokens = builtins.filter (x: x != "") (
        lib.splitString "-" (builtins.replaceStrings [ "_" " " ] [ "-" "-" ] lowerName)
      );
      strippedTokens = builtins.filter (x: x != "") (
        lib.splitString "-" (builtins.replaceStrings [ "_" " " ] [ "-" "-" ] stripped)
      );
      cleanMainProg = if mainProgram != null then safeString mainProgram else null;
      mainProgLower = if cleanMainProg != null then lib.toLower cleanMainProg else null;
      mainProgTokens =
        if mainProgLower != null then
          builtins.filter (x: x != "") (
            lib.splitString "-" (builtins.replaceStrings [ "_" " " ] [ "-" "-" ] mainProgLower)
          )
        else
          [ ];
      terminalKeywords =
        if terminal then
          [
            "terminal"
            "cli"
          ]
        else
          [ ];
      rawList = [
        lowerName
      ]
      ++ (lib.optional (stripped != lowerName) stripped)
      ++ tokens
      ++ strippedTokens
      ++ (lib.optional (mainProgLower != null && mainProgLower != lowerName) mainProgLower)
      ++ mainProgTokens
      ++ terminalKeywords;
    in
    lib.unique (builtins.filter (x: x != "" && x != null) rawList);

  # Split semicolon-separated list or ensure list of clean strings
  splitSemicolonList =
    val:
    if val == null then
      [ ]
    else if builtins.isList val then
      builtins.filter (x: x != "" && x != null) (
        builtins.map (x: if builtins.isString x then lib.strings.trim (safeString x) else x) val
      )
    else if builtins.isString val then
      let
        clean = safeString val;
        parts = lib.splitString ";" clean;
        trimmedParts = builtins.map lib.strings.trim parts;
      in
      builtins.filter (x: x != "") trimmedParts
    else
      [ ];

  # Parse [Desktop Entry] INI format from text
  parseDesktopIni =
    text:
    let
      clean = safeString text;
      lines = lib.splitString "\n" clean;
      hasSectionHeader = lib.hasInfix "[Desktop Entry]" clean;
      step =
        acc: line:
        let
          trimmed = lib.strings.trim line;
        in
        if trimmed == "" || lib.hasPrefix "#" trimmed then
          acc
        else if lib.hasPrefix "[" trimmed then
          if trimmed == "[Desktop Entry]" then acc // { inSection = true; } else acc // { inSection = false; }
        else if acc.inSection && lib.hasInfix "=" trimmed then
          let
            parts = lib.splitString "=" trimmed;
            key = lib.strings.trim (builtins.head parts);
            val = lib.concatStringsSep "=" (builtins.tail parts);
          in
          if acc.map ? ${key} then
            acc
          else
            acc
            // {
              map = acc.map // {
                ${key} = val;
              };
            }
        else
          acc;
      initAcc = {
        inSection = !hasSectionHeader;
        map = { };
      };
      result = builtins.foldl' step initAcc lines;
    in
    result.map;

  # Tokenize Exec command line handling spaces and quotes
  tokenizeExec =
    str:
    let
      clean = safeString str;
    in
    if clean == null || clean == "" then
      [ ]
    else
      let
        chars = lib.stringToCharacters clean;
        step =
          acc: char:
          if acc.inQuote then
            if char == acc.quoteChar then
              acc
              // {
                inQuote = false;
                quoteChar = "";
              }
            else
              acc
              // {
                current = acc.current + char;
              }
          else if char == "\"" || char == "'" then
            acc
            // {
              inQuote = true;
              quoteChar = char;
            }
          else if char == " " || char == "\t" || char == "\n" then
            if acc.current != "" then
              acc
              // {
                current = "";
                tokens = acc.tokens ++ [ acc.current ];
              }
            else
              acc
          else
            acc
            // {
              current = acc.current + char;
            };
        init = {
          inQuote = false;
          quoteChar = "";
          current = "";
          tokens = [ ];
        };
        res = builtins.foldl' step init chars;
      in
      if res.current != "" then res.tokens ++ [ res.current ] else res.tokens;

  # Extract desktop entry metadata from an item or makeDesktopItem derivation
  extractDesktopItem =
    item:
    if !builtins.isAttrs item then
      { }
    else
      let
        ini = if item ? text && builtins.isString item.text then parseDesktopIni item.text else { };
      in
      {
        desktopName =
          if item ? desktopName && item.desktopName != null && item.desktopName != "" then
            safeString item.desktopName
          else if ini ? Name && ini.Name != "" then
            safeString ini.Name
          else
            null;

        genericName =
          if item ? genericName && item.genericName != null && item.genericName != "" then
            safeString item.genericName
          else if item ? generic_name && item.generic_name != null && item.generic_name != "" then
            safeString item.generic_name
          else if ini ? GenericName && ini.GenericName != "" then
            safeString ini.GenericName
          else
            null;

        comment =
          if item ? comment && item.comment != null && item.comment != "" then
            sanitizeComment item.comment
          else if ini ? Comment && ini.Comment != "" then
            sanitizeComment ini.Comment
          else
            null;

        icon =
          if item ? icon && item.icon != null && item.icon != "" then
            safeString item.icon
          else if ini ? Icon && ini.Icon != "" then
            safeString ini.Icon
          else
            null;

        terminal =
          if item ? terminal && item.terminal != null then
            (item.terminal == true || item.terminal == "true")
          else if ini ? Terminal then
            (ini.Terminal == "true")
          else
            null;

        categories =
          if item ? categories && item.categories != null then
            splitSemicolonList item.categories
          else if ini ? Categories then
            splitSemicolonList ini.Categories
          else
            [ ];

        mimeTypes =
          if item ? mimeTypes && item.mimeTypes != null then
            splitSemicolonList item.mimeTypes
          else if item ? mimeType && item.mimeType != null then
            splitSemicolonList item.mimeType
          else if item ? mime_types && item.mime_types != null then
            splitSemicolonList item.mime_types
          else if ini ? MimeType then
            splitSemicolonList ini.MimeType
          else
            [ ];

        keywords =
          if item ? keywords && item.keywords != null then
            splitSemicolonList item.keywords
          else if ini ? Keywords then
            splitSemicolonList ini.Keywords
          else
            [ ];

        startupWMClass =
          if item ? startupWMClass && item.startupWMClass != null && item.startupWMClass != "" then
            safeString item.startupWMClass
          else if item ? startup_wm_class && item.startup_wm_class != null && item.startup_wm_class != "" then
            safeString item.startup_wm_class
          else if ini ? StartupWMClass && ini.StartupWMClass != "" then
            safeString ini.StartupWMClass
          else
            null;

        exec =
          if item ? exec && item.exec != null && item.exec != "" then
            safeString item.exec
          else if ini ? Exec && ini.Exec != "" then
            safeString ini.Exec
          else
            null;
      };

  # Helper to find raw desktop item from derivation or attrset
  getRawDesktopItem =
    name: drv:
    if !builtins.isAttrs drv then
      null
    else
      let
        candidates = [
          (drv.desktopItems or null)
          (drv.desktopItem or null)
          (if drv ? passthru then drv.passthru.desktopItems or null else null)
          (if drv ? passthru then drv.passthru.desktopItem or null else null)
        ];
        pickItem =
          c:
          if c == null then
            null
          else if builtins.isList c then
            let
              valid = builtins.filter (x: x != null && builtins.isAttrs x) c;
            in
            if valid == [ ] then
              null
            else
              let
                cleanName = safeString name;
                matching = builtins.filter (
                  x:
                  let
                    n = safeString (x.name or "");
                  in
                  n == "${cleanName}.desktop" || n == cleanName
                ) valid;
              in
              if matching != [ ] then builtins.head matching else builtins.head valid
          else if builtins.isAttrs c then
            c
          else
            null;
        found = builtins.filter (x: x != null) (builtins.map pickItem candidates);
      in
      if found != [ ] then builtins.head found else null;

  # Parse plain string: "pkg", "flake#pkg", or "github:org/repo#pkg"
  parseString =
    targetFlake: str:
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
        flakeOutput = if targetFlake != null then "${targetFlake}#${cleanStr}" else null;
      };

  # Check if a license indicates an unfree license
  isUnfreeLicense =
    lic:
    if builtins.isList lic then
      lib.any isUnfreeLicense lic
    else if builtins.isAttrs lic then
      !(lic.free or true)
    else if builtins.isString lic then
      lic == "unfree"
    else
      false;

  # Check if a derivation has an unfree license or meta.unfree set
  isDerivationUnfree =
    drv:
    if !builtins.isAttrs drv then
      false
    else
      (drv.meta.unfree or false) == true
      || (drv ? meta && drv.meta ? license && isUnfreeLicense drv.meta.license);

  # Parse derivation attributes safely without realizing the derivation
  parseDerivation =
    targetFlake: drv:
    let
      rawPname =
        drv.pname or (
          let
            parsed = builtins.parseDrvName (drv.name or "");
          in
          if parsed.name != "" then parsed.name else drv.name or "unknown-app"
        );
      name = safeString rawPname;
      rawDesktopItem = getRawDesktopItem name drv;
      desktopMeta = if rawDesktopItem != null then extractDesktopItem rawDesktopItem else { };

      desktopName =
        if
          desktopMeta ? desktopName && desktopMeta.desktopName != null && desktopMeta.desktopName != ""
        then
          desktopMeta.desktopName
        else if
          drv ? meta && drv.meta ? desktopName && drv.meta.desktopName != null && drv.meta.desktopName != ""
        then
          safeString drv.meta.desktopName
        else
          null;

      genericName = desktopMeta.genericName or null;

      comment =
        if desktopMeta ? comment && desktopMeta.comment != null then
          desktopMeta.comment
        else
          sanitizeComment (drv.meta.description or null);

      desktopMetaIcon =
        if desktopMeta ? icon && desktopMeta.icon != null && desktopMeta.icon != "" then
          desktopMeta.icon
        else if drv ? meta && drv.meta ? icon && drv.meta.icon != null && drv.meta.icon != "" then
          safeString drv.meta.icon
        else
          null;

      icon = desktopMetaIcon;

      terminal =
        if desktopMeta ? terminal && desktopMeta.terminal != null then
          desktopMeta.terminal
        else
          (drv.meta.terminal or false);

      categories = desktopMeta.categories or [ ];
      mimeTypes = desktopMeta.mimeTypes or [ ];
      keywords = desktopMeta.keywords or [ ];
      startupWMClass = desktopMeta.startupWMClass or null;
      exec = desktopMeta.exec or null;
      mainProgram = drv.meta.mainProgram or null;
      position = drv.meta.position or null;
    in
    {
      inherit
        name
        desktopName
        genericName
        comment
        icon
        desktopMetaIcon
        terminal
        categories
        mimeTypes
        keywords
        startupWMClass
        exec
        mainProgram
        position
        ;
      flakeOutput = if targetFlake != null then "${targetFlake}#${name}" else null;
      unfree = isDerivationUnfree drv;
    };

  # Main builder implementation
  mkLazyApp =
    arg:
    let
      appFollows = if builtins.isAttrs arg && arg ? follows then arg.follows else follows;

      appTargetFlake =
        if builtins.isAttrs arg && arg ? follows then
          (if arg.follows != null then resolveFlake arg.follows else null)
        else
          targetFlake;

      rawSpec =
        if builtins.isString arg then
          parseString appTargetFlake arg
        else if lib.isDerivation arg then
          parseDerivation appTargetFlake arg
        else if builtins.isAttrs arg then
          let
            drv =
              if arg ? package && lib.isDerivation arg.package then
                arg.package
              else if arg ? app && lib.isDerivation arg.app then
                arg.app
              else
                null;
            base =
              if drv != null then
                parseDerivation appTargetFlake drv
              else if arg ? flakeOutput && !(arg ? name) then
                parseString appTargetFlake arg.flakeOutput
              else
                { };
            explicitItem = getRawDesktopItem (arg.name or (base.name or "")) arg;
            explicitMeta = if explicitItem != null then extractDesktopItem explicitItem else { };
          in
          base // (lib.filterAttrs (n: v: v != null && v != [ ]) explicitMeta) // arg
        else
          throw "mkLazyApp: argument must be a derivation, string, or attribute set, but got ${builtins.typeOf arg}";

      name = safeString (rawSpec.name or (throw "mkLazyApp: unable to determine 'name' for application"));
      hasExplicitFlakeOutput =
        rawSpec ? flakeOutput
        && rawSpec.flakeOutput != null
        && lib.hasInfix "#" (safeString rawSpec.flakeOutput);
      flakeOutput =
        if hasExplicitFlakeOutput then
          safeString rawSpec.flakeOutput
        else if appTargetFlake != null then
          safeString (
            if rawSpec ? flakeOutput && rawSpec.flakeOutput != null then
              "${appTargetFlake}#${safeString rawSpec.flakeOutput}"
            else
              "${appTargetFlake}#${name}"
          )
        else
          throw "mkLazyApp: 'follows' argument is required to resolve package '${name}' (e.g. follows = inputs.nixpkgs)";
      desktopName =
        if rawSpec ? desktopName && rawSpec.desktopName != null && rawSpec.desktopName != "" then
          safeString rawSpec.desktopName
        else
          formatDesktopName name;
      genericName =
        if rawSpec ? genericName && rawSpec.genericName != null then
          safeString rawSpec.genericName
        else if rawSpec ? generic_name && rawSpec.generic_name != null then
          safeString rawSpec.generic_name
        else
          null;
      terminal = (rawSpec.terminal or false) == true;

      rawIcon =
        if rawSpec ? icon && rawSpec.icon != null && rawSpec.icon != "" then
          rawSpec.icon
        else if
          rawSpec ? desktopMetaIcon
          && rawSpec.desktopMetaIcon != null
          && rawSpec.desktopMetaIcon != ""
        then
          rawSpec.desktopMetaIcon
        else if rawSpec ? iconName && rawSpec.iconName != null && rawSpec.iconName != "" then
          rawSpec.iconName
        else if terminal then
          "utilities-terminal"
        else
          name;

      isCustomIcon = builtins.isPath rawIcon || lib.isDerivation rawIcon;
      iconName = safeString (rawSpec.iconName or name);

      customIconPkg =
        if isCustomIcon then
          pkgs.runCommand "${name}-icon" {
            src = rawIcon;
          } ''
            if [ -d "$src/share" ]; then
              mkdir -p "$out"
              cp -r "$src/share" "$out/"
            elif [ -d "$src" ]; then
              mkdir -p "$out/share/pixmaps"
              cp -r "$src"/. "$out/share/pixmaps/"
              for f in "$out/share/pixmaps"/*.svg; do
                if [ -f "$f" ]; then
                  mkdir -p "$out/share/icons/hicolor/scalable/apps"
                  cp "$f" "$out/share/icons/hicolor/scalable/apps/"
                fi
              done
            elif [ -f "$src" ]; then
              filename=$(basename "$src")
              ext="''${filename##*.}"
              if [ "$ext" = "$filename" ]; then
                if grep -q "<svg" "$src" 2>/dev/null; then
                  ext="svg"
                else
                  ext="png"
                fi
              fi
              mkdir -p "$out/share/pixmaps"
              cp "$src" "$out/share/pixmaps/${iconName}.$ext"
              if [ "$ext" = "svg" ] || [ "$ext" = "SVG" ]; then
                mkdir -p "$out/share/icons/hicolor/scalable/apps"
                cp "$src" "$out/share/icons/hicolor/scalable/apps/${iconName}.svg"
              fi
            else
              echo "Warning: icon source $src is not a file or directory" >&2
            fi
          ''
        else
          null;

      icon =
        if isCustomIcon then
          iconName
        else if builtins.isString rawIcon then
          safeString rawIcon
        else
          null;

      comment = sanitizeComment (rawSpec.comment or null);
      rawCategories = splitSemicolonList (rawSpec.categories or [ ]);
      categories =
        if rawCategories != [ ] then
          rawCategories
        else
          let
            position = rawSpec.position or (if rawSpec ? meta then rawSpec.meta.position or null else null);
            inferred = if position != null then inferCategoriesFromPosition position else [ ];
          in
          if inferred != [ ] then
            inferred
          else if terminal then
            [
              "System"
              "TerminalEmulator"
            ]
          else
            [ "Utility" ];
      mimeTypes = splitSemicolonList (
        rawSpec.mimeTypes or (rawSpec.mimeType or (rawSpec.mime_types or [ ]))
      );
      rawKeywords = splitSemicolonList (rawSpec.keywords or [ ]);
      keywords =
        if rawKeywords != [ ] then
          rawKeywords
        else
          buildDefaultKeywords {
            inherit name terminal;
            mainProgram =
              rawSpec.mainProgram or (if rawSpec ? meta then rawSpec.meta.mainProgram or null else null);
          };
      startupWMClass =
        if rawSpec ? startupWMClass && rawSpec.startupWMClass != null then
          safeString rawSpec.startupWMClass
        else if rawSpec ? startup_wm_class && rawSpec.startup_wm_class != null then
          safeString rawSpec.startup_wm_class
        else
          null;
      unfree = rawSpec.unfree or defaultAllowUnfree;
      impure = rawSpec.impure or false;
      flags = (lib.optionalString unfree " --unfree") + (lib.optionalString impure " --impure");

      wrapperScript = pkgs.writeShellScriptBin name ''
        exec ${lazyAppsPkg}/bin/nix-lazy-apps gui${flags} --flake-output ${lib.escapeShellArg flakeOutput} --name ${lib.escapeShellArg name} -- "$@"
      '';

      rawExec =
        if rawSpec ? exec && rawSpec.exec != null then
          if builtins.isString rawSpec.exec then safeString rawSpec.exec else rawSpec.exec
        else
          null;

      extraArgs =
        if rawExec == null || rawExec == "" then
          [ ]
        else if builtins.isList rawExec then
          rawExec
        else
          let
            tokens = tokenizeExec rawExec;
          in
          if tokens == [ ] then
            [ ]
          else
            let
              first = builtins.head tokens;
            in
            if lib.hasPrefix "-" first || lib.hasPrefix "%" first then tokens else builtins.tail tokens;

      hasFieldCode = lib.any (
        arg:
        lib.hasInfix "%U" arg || lib.hasInfix "%u" arg || lib.hasInfix "%F" arg || lib.hasInfix "%f" arg
      ) extraArgs;

      isGuiApp = !terminal;
      shouldAppendFieldCode = !hasFieldCode && (isGuiApp || mimeTypes != [ ]);
      finalExecArgs = if shouldAppendFieldCode then extraArgs ++ [ "%U" ] else extraArgs;
      formatArg =
        arg:
        let
          clean = safeString arg;
        in
        if lib.hasInfix " " clean && !(lib.hasPrefix "\"" clean && lib.hasSuffix "\"" clean) then
          "\"${clean}\""
        else
          clean;

      exec = lib.concatStringsSep " " (
        [ "${wrapperScript}/bin/${name}" ] ++ (builtins.map formatArg finalExecArgs)
      );

      spec = {
        inherit
          name
          flakeOutput
          desktopName
          genericName
          icon
          comment
          categories
          mimeTypes
          keywords
          startupWMClass
          terminal
          exec
          unfree
          impure
          ;
      }
      // lib.optionalAttrs (appFollows != null) { follows = appFollows; };

      knownDesktopItemArgs = [
        "name"
        "destination"
        "type"
        "desktopName"
        "genericName"
        "noDisplay"
        "comment"
        "icon"
        "onlyShowIn"
        "notShowIn"
        "dbusActivatable"
        "tryExec"
        "exec"
        "path"
        "terminal"
        "actions"
        "mimeTypes"
        "categories"
        "implements"
        "keywords"
        "startupNotify"
        "startupWMClass"
        "url"
        "prefersNonDefaultGPU"
        "singleMainWindow"
        "extraConfig"
      ];
      userDesktopConfig = (rawSpec.extraConfig or { }) // (rawSpec.extraDesktopConfig or { });
      knownUserAttrs = builtins.intersectAttrs (lib.genAttrs knownDesktopItemArgs (
        _: null
      )) userDesktopConfig;
      customUserAttrs = builtins.removeAttrs userDesktopConfig knownDesktopItemArgs;
      mergedExtraConfig = (userDesktopConfig.extraConfig or { }) // customUserAttrs;

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
        // lib.optionalAttrs (spec.genericName != null) { inherit (spec) genericName; }
        // lib.optionalAttrs (spec.icon != null) { inherit (spec) icon; }
        // lib.optionalAttrs (spec.comment != null) { inherit (spec) comment; }
        // lib.optionalAttrs (spec.mimeTypes != [ ]) { inherit (spec) mimeTypes; }
        // lib.optionalAttrs (spec.keywords != [ ]) { inherit (spec) keywords; }
        // lib.optionalAttrs (spec.startupWMClass != null) { inherit (spec) startupWMClass; }
        // knownUserAttrs
        // lib.optionalAttrs (mergedExtraConfig != { }) { extraConfig = mergedExtraConfig; }
      );
    in
    pkgs.symlinkJoin {
      inherit (spec) name;
      paths = [
        desktopItem
        wrapperScript
      ]
      ++ lib.optional (customIconPkg != null) customIconPkg
      ++ (rawSpec.extraPaths or [ ]);

      meta = (rawSpec.meta or { }) // {
        mainProgram = spec.name;
        description = spec.comment;
      };

      passthru = {
        inherit desktopItem wrapperScript spec;
      }
      // lib.optionalAttrs (customIconPkg != null) { inherit customIconPkg; }
      // (rawSpec.passthru or { });
    };
in
if args ? app then
  mkLazyApp args.app
else if args ? package then
  mkLazyApp args.package
else
  mkLazyApp
