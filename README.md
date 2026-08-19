# RAD Studio CLI

A command-line tool for discovering installed [Embarcadero RAD Studio](https://www.embarcadero.com/products/rad-studio) products (Delphi / C++Builder) and working with them — building projects with MSBuild, compiling resource files, and more — without having to open the IDE.

🚧 **This tool is currently under development.** Interfaces and commands may change.

## Overview

RAD Studio CLI reads the Windows Registry to find every installed RAD Studio / Delphi / C++Builder version on your machine (from classic Borland/CodeGear releases through the latest Embarcadero RAD Studio), and exposes that information through a simple CLI. It can also drive `MSBuild` to build `.dproj`, `.cbproj`, or `.groupproj` project files using the correct toolchain environment (`rsvars.bat` / `rsvars64.bat`) for a chosen version, architecture, and platform.

This makes it convenient to build Delphi/C++Builder projects from scripts, CI pipelines, AI agents, or any terminal.

## Features

- 🔍 **Discovery** — automatically detects all installed RAD Studio/Delphi/C++Builder versions from the registry.
- 🧭 **Version selection** — target an installation by product name (`RAD Studio 13`), codename (`Florence`, `Rio`, `Berlin`), or product version (`13`, `12`, `XE2`), or default to the latest installed version.
- 🛠️ **Build via MSBuild** — build `.dproj`/`.cbproj`/`.groupproj` files with a chosen configuration, architecture, and platform, optionally embedding version-info resources (company name, product version, copyright, etc.).
- 📦 **Resource compilation** — compile `.rc` resource script files to `.res` via `brcc32.exe`.
- ℹ️ **Product info** — print detailed information about installed products, including compiler/package versions, edition, personalities (Delphi/C++Builder), root directory, available architectures/platforms, and detected command-line compilers.
- 📌 **Self-install** — add (or remove) the CLI's directory to your user `PATH` so `radstudio` is available from any terminal.
- 🚧 **In development** — compile and install `.dpk` package files; configure IDE options such as environment variables and search paths.

## Requirements

- Windows (the tool relies on the Windows Registry and Windows-only APIs)
- A RAD Studio / Delphi / C++Builder installation registered under the current user
- [Rust toolchain](https://www.rust-lang.org/tools/install) if building from source

## Installation

You can download the latest release from the [Releases page](../../releases), or build it from source with Cargo:

```
git clone https://github.com/Delphier/radstudio.git
cd radstudio
cargo build --release
```

The resulting binary is `target/release/radstudio.exe`. Optionally, register it on your `PATH` so it can be run from anywhere:

```
radstudio self install
```

Run `radstudio self uninstall` to remove it from `PATH` again.

## Usage

```
radstudio [NAME] [COMMAND] [OPTIONS]
```

### Arguments

| Argument | Description                                                                                                                                                                                              |
| -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `[NAME]` | RAD Studio name or version to target, e.g. `13`, `XE2`, or `Florence`. Accepts a product name (`"RAD Studio 13.1"`), codename (`Rio`, `Berlin`), or version number. If omitted, the latest installed version is used. |

### Commands

| Command                           | Description                                                             |
| --------------------------------- | ----------------------------------------------------------------------- |
| `build` (alias `msbuild`)         | Build a project file (`*.dproj`, `*.cbproj`, `*.groupproj`) via MSBuild |
| `brcc` (alias `brcc32`)           | Compile a resource script file (`.rc`) into a `.res` file via `brcc32.exe` |
| `info`                            | Print installed RAD Studio product information                          |
| `self install` / `self uninstall` | Add or remove this tool from the user `PATH`                            |

Running `radstudio` with no command prints information about the selected installation (or all installations, if none is specified).

### Options

| Option                      | Description                                                                                                                             |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| `-a, --architecture <ARCH>` | Toolchain/IDE architecture: `IntelX86` (aliases `x86`, `32bit`) or `IntelX64` (aliases `x64`, `64bit`)                                  |
| `-p, --platform <PLATFORM>` | Target platform, e.g. `Win32`, `Win64`, `Win64x`, `WinARM64EC`, `OSX64`, `OSXARM64`, `Linux64`, `Android32`, `Android64`, `IOSDevice64` |
| `-h, --help`                | Print help                                                                                                                              |
| `-V, --version`             | Print version                                                                                                                           |

### Examples

Show all detected RAD Studio installations:

```
radstudio info
```

Show details for a specific version (by codename, product name, or number):

```
radstudio Florence info
radstudio 13 info
radstudio XE2 info
```

Build a project with the latest installed version:

```
radstudio build MyProject.dproj
```

Build a specific configuration/platform with a chosen RAD Studio version:

```
radstudio 13 build MyProject.dproj --config Release --platform Win64
```

Build the project using the 64-bit toolchain:

```
radstudio XE2 build MyProject.dproj --arch x64
```

Build and stamp the output with version-info resources:

```
radstudio build MyProject.dproj `
  --FileVersion "1.2.3" `
  --CompanyName "My Company" `
  --ProductName "My Product" `
  --LegalCopyright "Copyright © 2026 My Company"
```

Compile a resource script into a `.res` file:

```
radstudio brcc MyProject.rc
```

Compile with an explicit output path, using a specific RAD Studio version:

```
radstudio 13 brcc MyProject.rc MyProject.res
```

## Project structure

This is a Cargo workspace with two crates:

```
radstudio/
├── lib/    # `radstudio` library crate — registry discovery, product info, MSBuild/BRCC integration
└── cli/    # `radstudio-cli` crate — the `radstudio` command-line binary
```

## Contributing

Issues and pull requests are welcome. Since the project is still under active development, please open an issue to discuss significant changes before submitting a large PR.
