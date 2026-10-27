# nix-lazy-apps

> On-demand, lazy application launcher with live download and build progress for NixOS and Home Manager.

**`nix-lazy-apps`** allows you to declare desktop applications and CLI commands without downloading, compiling, or adding them to your system closure at switch time (`nixos-rebuild switch` or `home-manager switch`).

Instead of realizing entire software packages during system activation, `nix-lazy-apps` generates lightweight desktop entries (`.desktop`) and CLI wrapper scripts. When you launch an application—from your application launcher (Rofi, Wofi, GNOME, KDE, Hyprland, etc.) or terminal—`nix-lazy-apps` runs `nix run --log-format internal-json <flake_output>` and displays a responsive GUI showing real-time download/build progress, current activity, and a cancel button. Once the application is ready, the GUI automatically closes and hands over execution.

---

## Table of Contents

- [Features](#features)
- [How It Works](#how-it-works)
- [CLI Usage](#cli-usage)
- [Flake Integration](#flake-integration)
  - [Inputs](#inputs)
  - [Available Outputs](#available-outputs)
- [NixOS Configuration](#nixos-configuration)
- [Home Manager Configuration](#home-manager-configuration)
- [Supported `lazyPkgs` Formats](#supported-lazypkgs-formats)
  - [1. Derivations](#1-derivations)
  - [2. Plain Package Names](#2-plain-package-names)
  - [3. Flake References](#3-flake-references)
  - [4. Custom Attribute Sets](#4-custom-attribute-sets)
- [Standalone Helper Usage (`lib.mkLazyApp`)](#standalone-helper-usage-libmklazyapp)
- [Module Options Reference](#module-options-reference)

---

## Features

- **Instant Rebuilds:** Zero waiting for multi-gigabyte browser, game, or Electron downloads during system evaluation and activation.
- **Minimal Closures:** Prevents rarely-used or ad-hoc applications from bloating your generation's Nix store footprint.
- **Desktop & CLI Wrappers:** Automatically generates XDG-compliant `.desktop` items in `/share/applications` and binary wrappers in `/bin`.
- **Upstream Metadata Auto-Extraction:** Seamlessly inspects upstream `desktopItems` and `passthru.desktopItem` (e.g. `r2modman`) to extract names, descriptions, categories, icons, MIME types, search keywords, and window manager classes without realizing the underlying package closure into the Nix store.
- **Polished Desktop Entries:**
  - Automatic title casing for `desktopName` with known acronym preservation (`OBS`, `VLC`, `GIMP`, `IDE`, `CAD`, `UI`, `CLI`, etc.) and packaging suffix removal (`-bin`, `-wrapped`).
  - Strict XDG categories compliance with automatic category inference from Nixpkgs application paths (e.g. `pkgs/applications/graphics/...` -> `Graphics`), falling back to `Utility` or `TerminalEmulator`.
  - Search keywords auto-generation from application names and commands for launcher discovery (Rofi, Wofi, KRunner, GNOME, etc.).
- **Rich Icon Support:** Supports icon theme names, absolute paths, local file paths (`./icon.png`), SVG/PNG derivations (`pkgs.fetchurl`), or full icon directories, with automatic packaging into `$out/share/pixmaps` and `$out/share/icons/hicolor/scalable/apps`.
- **Terminal App Support:** Flags CLI/TUI applications (`terminal = true` or `meta.terminal`) to automatically launch inside the user's terminal emulator and attach CLI invocations directly.
- **Responsive Native GUI:** Built with Rust and `egui`, featuring:
  - Real-time download, fetch, and build progress bars.
  - Granular activity status (e.g., specific store path downloads, flake fetches).
  - A functional **Cancel** button to abort execution.
  - Automatic dismissal as soon as Nix hands off execution to the target app.
- **Flexible Package Declarations:** Works with raw derivations from `pkgs`, simple package name strings, flake URIs (`github:org/repo#app`), or detailed custom attribute sets.

---

## How It Works

Under standard NixOS or Home Manager configurations, adding `pkgs.r2modman` to `environment.systemPackages` or `home.packages` forces Nix to download or build the package and all its runtime dependencies before the rebuild finishes.

`nix-lazy-apps` bypasses this by:
1. Inspecting package derivations (such as `drv.pname` and `drv.meta`) while stripping Nix string contexts using `builtins.unsafeDiscardStringContext`.
2. Constructing a lightweight wrapper package containing only:
   - A `.desktop` entry pointing to `nix-lazy-apps gui --flake-output <flake-output> --name <name>`.
   - A shell script wrapper in `/bin/<name>` passing CLI arguments forward.
3. Because the string context is discarded, the derivation itself is **not** recorded as a runtime closure dependency.
4. When executed, `nix-lazy-apps` executes:
   ```bash
   nix run --log-format internal-json <flake_output>
   ```
   and parses Nix's structured JSON telemetry stream to update the GUI until the target application process starts.

---

## CLI Usage

The core binary can also be invoked manually from the command line:

```bash
nix-lazy-apps gui --flake-output <flake-output> --name <name>
```

### Options:

| Flag | Description |
| :--- | :--- |
| `--flake-output <string>` | The flake target to build/run (e.g., `nixpkgs#r2modman` or `github:user/repo#app`) |
| `--name <string>` | The display name of the application shown in the GUI modal |

### Example:

```bash
nix-lazy-apps gui --flake-output "nixpkgs#blender" --name "Blender"
```

---

## Flake Integration

### Inputs

Add `nix-lazy-apps` to your `flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    nix-lazy-apps = {
      url = "github:your-username/nix-lazy-apps"; # Adjust to actual repository location
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nix-lazy-apps, ... }: {
    # System configurations...
  };
}
```

### Available Outputs

- **`nixosModules.default`** (alias: `nixosModules.nix-lazy-apps`): NixOS system module.
- **`homeManagerModules.default`** (alias: `homeManagerModules.nix-lazy-apps`): Home Manager module.
- **`overlays.default`** (alias: `overlays.nix-lazy-apps`): Overlay exposing `pkgs.nix-lazy-apps`.
- **`lib.mkLazyApp`**: Standalone builder helper function to construct lazy application derivations.
- **`packages.<system>.default`** (alias: `packages.<system>.nix-lazy-apps`): The compiled `nix-lazy-apps` GUI executable.

---

## NixOS Configuration

Import the module in your NixOS configuration:

```nix
{ inputs, pkgs, ... }:

{
  imports = [
    inputs.nix-lazy-apps.nixosModules.default
  ];

  programs.nix-lazy-apps = {
    enable = true;
    follows = inputs.nixpkgs;
    lazyPkgs = with pkgs; [
      # 1. Package derivations (auto-extracts desktop items, icons, and categories)
      r2modman
      obs-studio

      # 2. Terminal / TUI applications (opens in terminal emulator)
      {
        package = btop;
        terminal = true;
      }

      # 3. Custom attrset with custom icon path or package
      {
        name = "custom-app";
        flakeOutput = "nixpkgs#custom-app";
        icon = ./icons/custom-app.png;
      }
    ];
  };
}
```

---

## Home Manager Configuration

Import the module in your Home Manager configuration:

```nix
{ inputs, pkgs, ... }:

{
  imports = [
    inputs.nix-lazy-apps.homeManagerModules.default
  ];

  programs.nix-lazy-apps = {
    enable = true;
    follows = inputs.nixpkgs;
    lazyPkgs = with pkgs; [
      # Derivations, terminal apps, strings, and custom attrsets:
      r2modman
      {
        package = btop;
        terminal = true;
      }
      {
        name = "custom-app";
        flakeOutput = "nixpkgs#custom-app";
        icon = ./icons/custom-app.png;
      }
    ];
  };
}
```

---

## Supported `lazyPkgs` Formats

The `lazyPkgs` option accepts a list containing any combination of the following formats:

### 1. Derivations

Pass existing package derivations directly from `pkgs`:

```nix
programs.nix-lazy-apps.lazyPkgs = with pkgs; [
  r2modman
  blender
  gimp
  obs-studio
  vlc
];
```

- **Upstream `desktopItems` extraction:** Automatically detects and extracts metadata from `drv.desktopItems` or `drv.passthru.desktopItem` (as seen in packages like `r2modman` or IDEs), preserving upstream comments, icons, categories, MIME types, search keywords, and window manager classes.
- **Title casing & acronym preservation:** Formats application names into clean, readable titles while preserving recognized industry acronyms (e.g. `OBS Studio`, `VLC`, `GIMP`, `IDE`, `2D`, `3D`, `CAD`, `PDF`, `SQL`).
- **Packaging suffix removal:** Strips common packaging suffixes such as `-bin`, `-wrapped`, and `-unwrapped` from generated display names.
- **XDG categories & inference:** Falls back to registered Freedesktop categories by inspecting the package's Nixpkgs position path (e.g. `pkgs/applications/graphics/...` -> `Graphics`, `pkgs/applications/audio/...` -> `AudioVideo`, etc.), defaulting to `[ "Utility" ]` or `[ "System" "TerminalEmulator" ]`.
- **Search keywords:** Automatically generates launcher search keywords from application names and commands so applications are instantly discoverable in Rofi, Wofi, KRunner, and GNOME Shell.
- **Zero store realization:** Evaluates derivations purely from Nix metadata and strips Nix string contexts using `builtins.unsafeDiscardStringContext`—the heavy package is **never downloaded or built into your system closure** at rebuild time.

### 2. Plain Package Names

Specify plain strings for attributes available in `follows`:

```nix
programs.nix-lazy-apps.lazyPkgs = [
  "r2modman"
  "obsidian"
  "audacity"
];
```

- Resolves against `follows` (e.g. `<follows>#<name>`).
- Binary wrapper and `.desktop` name will use the provided name.

### 3. Flake References

Specify explicit flake outputs from Nixpkgs or any external repository:

```nix
programs.nix-lazy-apps.lazyPkgs = [
  "nixpkgs#r2modman"
  "nixpkgs#legacyPackages.x86_64-linux.libreoffice-fresh"
  "github:hyprwm/hyprpicker#hyprpicker"
];
```

- Any string containing `#` is treated as a fully-qualified flake reference.
- The command name is automatically derived from the attribute name following the `#`.

### 4. Custom Attribute Sets

For full control over metadata, categories, icons, terminal launch behavior, and command arguments:

```nix
programs.nix-lazy-apps.lazyPkgs = [
  # 1. Real-world mod manager with upstream desktop item overrides
  {
    name = "r2modman";
    desktopName = "R2ModMan";
    genericName = "Mod Manager";
    flakeOutput = "nixpkgs#r2modman";
    icon = "r2modman";
    comment = "Mod manager for Thunderstore";
    categories = [ "Game" ];
    mimeTypes = [ "x-scheme-handler/ror2mm" ];
    keywords = [ "launcher" "mod manager" "thunderstore" ];
  }

  # 2. Terminal / TUI application launching inside a terminal emulator
  {
    package = pkgs.btop;
    terminal = true;
  }

  # 3. Custom application with local SVG/PNG icon file or fetchurl derivation
  {
    name = "my-studio";
    flakeOutput = "nixpkgs#obs-studio";
    desktopName = "Custom Studio";
    icon = ./assets/studio-icon.svg; # Path literal or pkgs.fetchurl derivation
    iconName = "my-studio-icon";
    categories = [ "AudioVideo" "Recorder" ];
  }

  # 4. Complex CLI application with custom exec arguments and field codes
  {
    name = "my-editor";
    flakeOutput = "nixpkgs#neovim";
    terminal = true;
    exec = "nvim --clean %F";
  }
];
```

#### Supported Icon Types:

`nix-lazy-apps` supports multiple icon formats through the `icon` attribute:
1. **Themed Icon Name (String):** E.g. `"firefox"` or `"utilities-terminal"`. Refers to icons provided by your desktop icon theme.
2. **Absolute Path (String):** E.g. `"/opt/myapp/share/icons/app.png"`. Referenced directly in `Icon=`.
3. **Local Path Literal (Path):** E.g. `./icons/app.png` or `./icon.svg`. The file is automatically copied into the wrapper package under `$out/share/pixmaps` and scalable SVGs under `$out/share/icons/hicolor/scalable/apps`.
4. **Image Derivation (Package):** E.g. `pkgs.fetchurl { url = "..."; hash = "..."; }` or `pkgs.runCommand`. Automatically packaged into standard XDG pixmaps and scalable icon theme locations.
5. **Icon Directory (Package / Path):** A derivation containing `$out/share/icons/...` or a directory of icon files. Installed alongside the wrapper package.

#### Supported Attrset Fields:

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `package` / `app` | derivation | `null` | Derivation to extract metadata and default settings from (inspects `desktopItems`, `passthru.desktopItem`, and `meta` without store realization). |
| `name` | string | *(derived)* | Base name and command placed in `/bin`. |
| `desktopName` | string | *(title-cased name)* | Human-readable name displayed in desktop application menus (preserves acronyms like `OBS`, `VLC`, `GIMP`, `IDE`). |
| `genericName` | string | `null` | Generic description (e.g., `"Web Browser"`, `"Mod Manager"`, `"Text Editor"`). |
| `flakeOutput` | string | `"${follows}#${name}"` | Flake URI passed to `nix run`. |
| `icon` | string / path / derivation | `name` (or `"utilities-terminal"`) | Icon name, absolute path string, path literal (`./icon.png`), SVG/PNG derivation (`pkgs.fetchurl`), or full icon directory. |
| `iconName` | string | `name` | Base file name to use when packaging custom file or derivation icons. |
| `comment` | string | `drv.meta.description` | Tooltip / description in the `.desktop` file (sanitized to a single line). |
| `categories` | list of strings | *(inferred or `[ "Utility" ]`)* | XDG desktop categories (e.g., `[ "Utility" "Development" ]`, inferred from Nixpkgs path, or `[ "System" "TerminalEmulator" ]`). |
| `mimeTypes` | list of strings | `[ ]` | Supported MIME types and URI schemes (e.g., `[ "x-scheme-handler/ror2mm" "text/plain" ]`). |
| `keywords` | list of strings | *(auto-generated)* | Search keywords for desktop application launchers (Rofi, KRunner, GNOME, etc.). |
| `startupWMClass` | string | `null` | Window manager class for grouping windows under docks and taskbars. |
| `terminal` | boolean | `drv.meta.terminal or false` | Whether the application runs inside a terminal window (launchers open a terminal emulator; CLI runs directly). |
| `exec` | string | `"${wrapperScript}/bin/${name} %U"` | Custom `Exec=` line for the desktop entry. CLI arguments and field codes (`%U`, `%f`) are preserved and routed to the wrapper script. |
| `unfree` | boolean | `defaultAllowUnfree` | Whether to allow unfree licenses when building/running this app. |
| `impure` | boolean | `false` | Whether to allow impure evaluation (`--impure`). |
| `follows` | flake input | `null` | Optional per-application flake input override. |
| `extraDesktopConfig` / `extraConfig` | attrset | `{ }` | Additional attributes forwarded to `pkgs.makeDesktopItem`. |
| `extraPaths` | list of paths | `[ ]` | Extra derivations or directories symlinked alongside the wrapper. |

---

## Standalone Helper Usage (`lib.mkLazyApp`)

If you prefer not to use the NixOS or Home Manager modules, you can use the `lib.mkLazyApp` helper directly in any Nix expression:

```nix
{ inputs, pkgs, ... }:

let
  # Initialize helper with pkgs and follows
  mkLazyApp = inputs.nix-lazy-apps.lib.mkLazyApp {
    inherit pkgs;
    follows = inputs.nixpkgs;
  };
in
{
  environment.systemPackages = [
    # Derivation
    (mkLazyApp pkgs.r2modman)

    # String
    (mkLazyApp "kdenlive")

    # Flake reference
    (mkLazyApp "github:user/custom-app#app")

    # Custom attribute set
    (mkLazyApp {
      name = "r2modman";
      desktopName = "R2ModMan Mod Manager";
      flakeOutput = "nixpkgs#r2modman";
      icon = "r2modman";
      comment = "Mod manager for Thunderstore";
      categories = [ "Game" ];
    })
  ];
}
```

You can also pass `app` directly into the function invocation:

```nix
inputs.nix-lazy-apps.lib.mkLazyApp {
  inherit pkgs;
  follows = inputs.nixpkgs;
  app = pkgs.r2modman;
}
```

---

## Module Options Reference

The NixOS and Home Manager modules provide the following options under `programs.nix-lazy-apps`:

### `programs.nix-lazy-apps.enable`
- **Type:** `boolean`
- **Default:** `false`
- **Description:** Whether to enable `nix-lazy-apps`. When enabled, installs the `nix-lazy-apps` package and all generated wrappers into the system closure (`environment.systemPackages` for NixOS, `home.packages` for Home Manager).

### `programs.nix-lazy-apps.lazyPkgs`
- **Type:** `listOf (derivation | string | attributeSet)`
- **Default:** `[ ]`
- **Description:** List of applications to generate lazy wrappers for. Accepts derivations, package name strings, flake output strings, or detailed attribute sets.

### `programs.nix-lazy-apps.follows`
- **Type:** `flake input | path | string`
- **Default:** `null`
- **Description:** The flake input to follow for resolving packages (e.g., `inputs.nixpkgs`). Ensures lazy applications build and run against the exact same channel and locked revision as the host flake that includes `nix-lazy-apps`.

### `programs.nix-lazy-apps.package`
- **Type:** `package`
- **Default:** `self.packages.${pkgs.stdenv.hostPlatform.system or pkgs.system}.default` or `pkgs.nix-lazy-apps`
- **Description:** The `nix-lazy-apps` executable package used to display the progress GUI.
