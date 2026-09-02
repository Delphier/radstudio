![RAD Studio CLI](assets/radstudio.png)

# RAD Studio CLI

A command-line tool for discovering installed [Embarcadero RAD Studio](https://www.embarcadero.com/products/rad-studio) products (Delphi / C++Builder) and working with them — building projects with MSBuild, compiling resource files, managing IDE environment variables and search paths, and more — without having to open the IDE.

🚧 **This tool is currently under development.** Interfaces and commands may change.

## Overview

RAD Studio CLI reads the Windows Registry to find every installed RAD Studio / Delphi / C++Builder version on your machine (from classic Borland/CodeGear releases through the latest Embarcadero RAD Studio), and exposes that information through a simple CLI. It can also drive `MSBuild` to build `.dproj`, `.cbproj`, or `.groupproj` project files using the correct toolchain environment (`rsvars.bat` / `rsvars64.bat`) for a chosen version, architecture, and platform, invoke the Delphi command-line compilers (`DCC32.exe`, `DCC64.exe`, `DCCARM64EC.exe`), and can read or update the IDE's registry-backed environment variables and search paths.

This makes it convenient to build Delphi/C++Builder projects and manage IDE configuration from scripts, CI pipelines, AI agents, or any terminal.

## Features

- 🔍 **Discovery** — automatically detects all installed RAD Studio/Delphi/C++Builder versions from the registry.
- 🧭 **Version selection** — target an installation by product name (`RAD Studio 13`), codename (`Florence`, `Rio`, `Berlin`), or product version (`13`, `12`, `XE2`), or default to the latest installed version.
- 🛠️ **Build via MSBuild** — build `.dproj`/`.cbproj`/`.groupproj` files with a chosen configuration, architecture, and platform, optionally embedding version-info resources (company name, product version, copyright, etc.).
- 🧱 **Build via bds.exe** — build the same project files through `bds.exe` instead of MSBuild (same options as `build`), which avoids the "This version of the product does not support command-line compiling" prompt shown by Community/Trial editions.
- 🧮 **Direct compiler invocation** — compile Delphi files straight through `DCC32.exe`/`DCC64.exe`/`DCCARM64EC.exe` (`dcc32`/`dcc64`/`dccarm64ec` commands), with options for conditional defines, unit/resource/include search directories, output directories, and passing through raw compiler switches.
- 📦 **Resource compilation** — compile `.rc` resource script files to `.res` via `brcc32.exe`.
- ⚙️ **IDE environment variables** — view, set, or remove environment variables stored per-architecture for a RAD Studio installation.
- 🧩 **Search path management** — view, add, insert, or remove entries in the IDE's environment `PATH`, Library path, and Browsing path, per architecture/platform.
- ℹ️ **Product info** — print detailed information about installed products, including compiler/package versions, edition, personalities (Delphi/C++Builder), root directory, available architectures/platforms, and detected command-line compilers.
- 📌 **Self-install** — add (or remove) the CLI's directory to your user `PATH` so `radstudio` is available from any terminal.
- 🚧 **In development** — compile and install `.dpk` package files.

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

| Command                                     | Description                                                              |
| -------------------------------------------- | ------------------------------------------------------------------------ |
| `build` (alias `msbuild`)                    | Build a project file (`*.dproj`, `*.cbproj`, `*.groupproj`) via MSBuild  |
| `bds`                                        | Build a project file via `bds.exe` (same options as `build`); avoids the command-line compiling restriction on Community/Trial editions |
| `dcc32`                                      | Compile Delphi files for Win32 via `DCC32.exe`                  |
| `dcc64`                                      | Compile Delphi files for Win64 via `DCC64.exe`                  |
| `dccarm64ec`                                 | Compile Delphi files for WinARM64EC via `DCCARM64EC.exe`        |
| `brcc` (alias `brcc32`)                      | Compile a resource script file (`.rc`) into a `.res` file via `brcc32.exe` |
| `env`                                        | View, set, or remove IDE environment variables                          |
| `env-path` (alias `path`)                     | View, add, insert, or remove entries in the IDE environment `PATH`      |
| `library-path` (aliases `lib-path`, `libpath`) | View, add, insert, or remove entries in the IDE Library path           |
| `browsing-path`                               | View, add, insert, or remove entries in the IDE Browsing path           |
| `info`                                       | Print installed RAD Studio product information                          |
| `self install` / `self uninstall`            | Add or remove this tool from the user `PATH`                            |

Running `env`, `envpath`, `librarypath`, or `browsingpath` with no subcommand prints the current values. Each supports its own subcommands:

- `env` — `set`/`add <NAME> <VALUE>` to set a variable, `remove`/`rm`/`delete`/`del <NAME>` to remove one.
- `envpath`, `librarypath`, `browsingpath` — `add`/`push`/`append <ITEM>` to append an entry (skipped if it already exists), `insert <ITEM>` to prepend an entry, `remove`/`rm`/`delete`/`del <ITEM>` to remove one.

`dcc32`, `dcc64`, and `dccarm64ec` take a source `<FILE>` plus compiler options: `--no-config` (skip the default `dcc*.cfg`), `-D/--define <NAME>` (repeatable), `--unit-search-dirs`/`--resource-search-dirs`/`--include-search-dirs <DIRS>`, `-B/--build` (rebuild all units), `-Q/--quiet`, `--output-dir`/`--unit-output-dir`/`--package-bpl-output-dir`/`--package-dcp-output-dir <DIR>`, C++Builder-related flags (`--cpp`, `--cpp-win64x`, `--cpp-bpi-output-dir`, `--cpp-hpp-output-dir`, `--cpp-obj-output-dir`), and `--options <RAW>` to append any additional raw compiler switches verbatim.

### Options

| Option                      | Description                                                                                                                             |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `-a, --architecture <ARCH>` | Toolchain/IDE architecture: `IntelX86` (aliases `x86`, `32bit`) or `IntelX64` (aliases `x64`, `64bit`)                                  |
| `-p, --platform <PLATFORM>` | Target platform, e.g. `Win32`, `Win64`, `Win64x`, `WinARM64EC`, `OSX64`, `OSXARM64`, `Linux64`, `Android32`, `Android64`, `IOSDevice64` |
| `-h, --help`                | Print help                                                                                                                                |
| `-V, --version`             | Print version                                                                                                                             |

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
radstudio XE8 build MyProject.dproj --arch x64
```

Build and stamp the output with version-info resources:

```
radstudio build MyProject.dproj `
  --FileVersion "1.2.3" `
  --CompanyName "My Company" `
  --ProductName "My Product" `
  --LegalCopyright "Copyright © 2026 My Company"
```

Build with `bds.exe` instead of MSBuild (useful on Community/Trial editions that block command-line compiling via MSBuild):

```
radstudio bds MyProject.dproj --config Release --platform Win64
```

Compile a Delphi source file directly with `DCC32.exe`:

```
radstudio dcc32 MyProject.dpr
```

Compile for Win64 with a conditional define and a custom unit output directory:

```
radstudio dcc64 MyProject.dpr -d DEBUG --unit-output-dir .\dcu
```

Compile for WinARM64EC using a specific RAD Studio version, passing raw extra options through:

```
radstudio 13 dccarm64ec MyProject.dpr --options "-W-SYMBOL_DEPRECATED"
```

Compile a resource script into a `.res` file:

```
radstudio brcc MyProject.rc
```

Compile with an explicit output path, using a specific RAD Studio version:

```
radstudio 13 brcc MyProject.rc MyProject.res
```

Show IDE environment variables for the latest installed version:

```
radstudio env
```

Set an IDE environment variable:

```
radstudio env set MY_VAR "some value"
```

Remove an IDE environment variable:

```
radstudio env remove MY_VAR
```

Append a directory to the IDE's environment `PATH`:

```
radstudio envpath add "C:\Tools\bin"
```

Show the Library path for a specific platform:

```
radstudio 13 librarypath --platform Win64
```

Insert an entry at the front of the Browsing path:

```
radstudio browsingpath insert "C:\MyLib\Include"
```

## Project structure

This is a Cargo workspace with two crates:

```
crates/
├── radstudio/        # library crate — registry discovery, product info, tools integration
└── radstudio-cli/    # cli crate — the `radstudio` command-line binary
```

## Contributing

Issues and pull requests are welcome. Since the project is still under active development, please open an issue to discuss significant changes before submitting a large PR.
