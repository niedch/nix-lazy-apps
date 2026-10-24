{
  lib ? null,
  ...
}:

let
  hasPrefix =
    if lib != null && lib ? hasPrefix then
      lib.hasPrefix
    else
      prefix: str: builtins.substring 0 (builtins.stringLength prefix) str == prefix;

  safeString = s: builtins.unsafeDiscardStringContext (builtins.toString s);

  resolveFlake =
    input:
    if input == null then
      null
    else if builtins.isString input then
      let
        s = safeString input;
      in
      if (hasPrefix "/" s || hasPrefix "./" s) && !hasPrefix "path:" s then "path:${s}" else s
    else if builtins.isPath input then
      let
        p = safeString input;
      in
      if hasPrefix "path:" p then p else "path:${p}"
    else if builtins.isAttrs input then
      let
        p = safeString (input.outPath or (builtins.toString input));
      in
      if hasPrefix "path:" p then p else "path:${p}"
    else
      throw "resolveFlake: unsupported input type '${builtins.typeOf input}'";
in
resolveFlake
