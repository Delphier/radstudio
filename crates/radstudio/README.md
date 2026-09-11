# radstudio

A Rust library for discovering installed [Embarcadero RAD Studio](https://www.embarcadero.com/products/rad-studio) products (Delphi / C++Builder) on Windows, inspecting their toolchains, and driving them programmatically — building projects with MSBuild or bds.exe, compiling resource scripts, and more.

This is the library crate behind [`radstudio-cli`](https://github.com/Delphier/radstudio), the `radstudio` command-line tool. It can also be used directly in your own Rust programs, scripts, build tools, or CI/agent integrations.

## Features

- 🔍 **Discovery** — reads the Windows Registry to find every installed RAD Studio / Delphi / C++Builder version, from classic Borland/CodeGear releases through the latest Embarcadero RAD Studio.
- ℹ️ **Product info** — query version, compiler version, package version, edition, personalities (Delphi/C++Builder), code name (e.g. `Rio`, `Sydney`, `Florence`), root directory, and more for each installation.
- 🧭 **Architecture & platform detection** — determine which toolchain architectures (`x86`/`x64`) and target platforms (`Win32`, `Win64`, `OSX64`, `Android64`, `IOSDevice64`, ...) are available for a given installation.
- 🛠️ **MSBuild integration** — build `.dproj` / `.cbproj` / `.groupproj` files using the correct `rsvars.bat` / `rsvars64.bat` environment for a chosen version and architecture, optionally embedding version-info resources (company name, product version, copyright, etc.).
- 📦 **Resource compilation** — compile `.rc` resource script files to `.res` via `brcc32.exe`.
- 🧩 **Command-line tool detection** — locate compilers such as `DCC32`, `DCC64`, `BCC64X`, `DCCOSX64`, `DCCLINUX64`, `DCCAARM64`, `DCCIOSARM64`, and `BRCC32` for a given installation and architecture.

## Requirements

- **Windows** — this crate relies on the Windows Registry and Windows-only APIs and will not build on other platforms.
- A RAD Studio / Delphi / C++Builder installation registered under the current user.

## Usage

### Discover installed RAD Studio versions

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let installs = radstudio::find()?;
    println!("Found {} installation(s)", installs.count());

    for install in installs.iter() {
        let info = install.product_info();
        println!("{} ({})", info.full_name(), info.version());
    }

    Ok(())
}
```

### Look up a specific installation

Installations can be matched by product name, code name, or version number:

```rust
let installs = radstudio::find()?;

// By code name, product version, or full/display name — case-insensitive.
if let Some(install) = installs.find_by_name("Florence") {
    println!("{}", install.product_info());
}

// Fall back to the most recently installed version.
if let Some(install) = installs.latest() {
    println!("Latest: {}", install.product_info().full_name());
}
```

`ProductInfo` (returned by `product_info()`) also implements `Display`, so `println!("{info}")` prints a formatted table of everything the crate knows about the installation — version numbers, edition, personalities, root directory, per-architecture paths, and detected command-line tools.

### Build a project with MSBuild

```rust
use radstudio::msbuild::Options;
use std::path::PathBuf;

let installs = radstudio::find()?;
let install = installs.latest().expect("no RAD Studio installation found");

// `None` selects the first available toolchain architecture automatically.
let msbuild = install.msbuild(&None).expect("MSBuild environment not available");

let options = Options {
    file: PathBuf::from("MyProject.dproj"),
    config: Some("Release".to_string()),
    version_info: Default::default(),
};

let status = msbuild.execute(&Some(radstudio::Platform::Win64), &options)?;
assert!(status.success());
```

### Compile a resource script

```rust
use radstudio::brcc::Options;
use std::path::PathBuf;

let installs = radstudio::find()?;
let install = installs.latest().expect("no RAD Studio installation found");

let brcc = install.brcc32(&None).expect("brcc32.exe not found");
let status = brcc.execute(&Options {
    input_rc: PathBuf::from("MyProject.rc"),
    output_res: Some(PathBuf::from("MyProject.res")),
})?;
assert!(status.success());
```

## API overview

| Item                                       | Description                                                                                     |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------- |
| `find()`                                   | Scans the registry and returns all detected installations as `Installations`.                     |
| `Installations`                            | A collection of `Installation`s, sorted by version; supports `iter()`, `find_by_name()`, `latest()`, `count()`. |
| `Installation`                             | A single RAD Studio installation; exposes `product_info()`, `msbuild()`, `brcc32()`.               |
| `ProductInfo`                              | Metadata about an installation: version, compiler/package version, edition, personalities, root directory, architectures, platforms, and command-line tools. Implements `Display` as a formatted table. |
| `Architecture`                             | Toolchain/IDE architecture: `X86` or `X64`.                                                        |
| `Platform`                                 | Target platform, e.g. `Win32`, `Win64`, `Win64x`, `WinARM64EC`, `OSX64`, `OSXARM64`, `Linux64`, `Android32`, `Android64`, `IOSDevice64`. |
| `Personality`                              | `Delphi` or `CBuilder`, indicating which language(s) an installation supports.                     |
| `Edition`                                  | Product edition, e.g. `Community`.                                                                 |
| `CommandLineTool`                          | Known compiler executables, e.g. `DCC32`, `DCC64`, `BCC64X`, `BRCC32`.                             |
| `msbuild::MsBuild`, `msbuild::Options`     | Drives `MSBuild.exe` against a `.dproj`/`.cbproj`/`.groupproj` file using the correct `rsvars` environment. |
| `brcc::Brcc`, `brcc::Options`               | Drives `brcc32.exe` to compile a `.rc` file into a `.res` file.                                    |

## Related

- [`radstudio-cli`](https://github.com/Delphier/radstudio) — the command-line tool built on top of this library.

## Contributing

Issues and pull requests are welcome on the [main repository](https://github.com/Delphier/radstudio).
