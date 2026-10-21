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
    lazyPkgs = with pkgs; [
      r2modman
      # strings and custom attrsets also work!
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
    lazyPkgs = with pkgs; [
      r2modman
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
];
```

- Automatically extracts `pname` (or `name`), `meta.desktopName`, `meta.description`, and icon names.
- Discards string context so the derivation is **never realized into your system closure** at switch time.
- Resolves flake target as `${defaultFlake}#${pname}` (defaults to `nixpkgs#<pname>`).

### 2. Plain Package Names

Specify plain strings for attributes available in your `defaultFlake`:

```nix
programs.nix-lazy-apps.lazyPkgs = [
  "r2modman"
  "obsidian"
  "audacity"
];
```

- Resolves to `${defaultFlake}#<name>` (e.g., `nixpkgs#r2modman`).
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

For full control over metadata, categories, icons, and command behavior:

```nix
programs.nix-lazy-apps.lazyPkgs = [
  {
    name = "r2modman";
    desktopName = "R2ModMan Mod Manager";
    flakeOutput = "nixpkgs#r2modman";
    icon = "r2modman";
    comment = "Mod manager for Thunderstore";
    categories = [ "Game" ];
  }
  {
    name = "custom-cli-tool";
    flakeOutput = "github:user/tools#custom-cli";
    terminal = true;
  }
];
```

#### Supported Attrset Fields:

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `name` | string | *(derived)* | Name of the binary placed in `/bin`. |
| `desktopName` | string | `name` | Human-readable name displayed in desktop application menus. |
| `flakeOutput` | string | `"${defaultFlake}#${name}"` | Flake URI passed to `nix run`. |
| `icon` | string | `name` | Icon identifier or path for the `.desktop` file. |
| `comment` | string | `null` | Tooltip / description in the `.desktop` file. |
| `categories` | list of strings | `[ ]` | XDG desktop categories (e.g., `[ "Development" "IDE" ]`). |
| `terminal` | boolean | `false` | Whether the application runs inside a terminal window. |
| `exec` | string | *(default wrapper call)* | Custom `Exec=` line for the desktop entry. |
| `extraDesktopConfig` | attrset | `{ }` | Additional attributes forwarded to `pkgs.makeDesktopItem`. |
| `extraPaths` | list of paths | `[ ]` | Extra derivations symlinked alongside the wrapper. |

---

## Standalone Helper Usage (`lib.mkLazyApp`)

If you prefer not to use the NixOS or Home Manager modules, you can use the `lib.mkLazyApp` helper directly in any Nix expression:

```nix
{ inputs, pkgs, ... }:

let
  # Initialize helper with pkgs
  mkLazyApp = inputs.nix-lazy-apps.lib.mkLazyApp pkgs;

  # Or configure with custom defaults
  # mkLazyApp = inputs.nix-lazy-apps.lib.mkLazyApp {
  #   inherit pkgs;
  #   defaultFlake = "nixpkgs";
  # };
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

### `programs.nix-lazy-apps.defaultFlake`
- **Type:** `string`
- **Default:** `"nixpkgs"`
- **Description:** Default flake URI used when resolving packages specified by name (e.g., `"r2modman"` becomes `"nixpkgs#r2modman"`).

### `programs.nix-lazy-apps.package`
- **Type:** `package`
- **Default:** `self.packages.${pkgs.system}.default` or `pkgs.nix-lazy-apps`
- **Description:** The `nix-lazy-apps` executable package used to display the progress GUI.
