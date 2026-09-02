mod env;
mod paths;
mod self_;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use radstudio::{
    Architecture, Architectures, CommandLineTool, Installation, Installations, Platform, Platforms,
};
use std::sync::OnceLock;

const APP_NAME: &'static str = "RAD Studio CLI";
static INSTALLATIONS: OnceLock<Installations> = OnceLock::new();

fn installations() -> &'static Installations {
    INSTALLATIONS.get_or_init(|| radstudio::find().unwrap())
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    match &app.subcmd {
        Some(Cmd::Build { options }) => {
            app.installation()?
                .msbuild(&app.global.architecture)
                .context("MSBuild.exe not found")?
                .execute(&app.global.architecture, &app.global.platform, &options)?;
        }
        Some(Cmd::Bds { options }) => {
            app.installation()?
                .bds(&app.global.architecture)
                .context(format!(
                    "bds.exe not found{}",
                    app.global
                        .architecture
                        .as_ref()
                        .map(|a| format!(" ({} not installed)", a.ide_name()))
                        .unwrap_or_default()
                ))?
                .execute(&app.global.architecture, &app.global.platform, &options)?;
        }
        Some(Cmd::Dcc32 { options }) => {
            app.dcc_execute(&CommandLineTool::DCC32, &options)?;
        }
        Some(Cmd::Dcc64 { options }) => {
            app.dcc_execute(&CommandLineTool::DCC64, &options)?;
        }
        Some(Cmd::Dccarm64ec { options }) => {
            app.dcc_execute(&CommandLineTool::DCCARM64EC, &options)?;
        }
        Some(Cmd::Brcc { options }) => {
            app.installation()?
                .brcc32(&app.global.architecture)
                .context(err_clt_not_found(&CommandLineTool::BRCC32))?
                .execute(&options)?;
        }
        Some(Cmd::Env { subcmd }) => {
            env::EnvCmd::execute(subcmd, app.installation()?, app.ide_architectures()?)?;
        }
        Some(Cmd::EnvPath { subcmd }) => {
            env::path::execute(subcmd, app.installation()?, app.ide_architectures()?)?;
        }
        Some(Cmd::LibraryPath { subcmd }) => {
            paths::PathsCmd::execute(
                "Library Path",
                Installation::LIBRARY_PATH,
                subcmd,
                app.installation()?,
                app.platforms()?,
            )?;
        }
        Some(Cmd::BrowsingPath { subcmd }) => {
            paths::PathsCmd::execute(
                "Browsing Path",
                Installation::BROWSING_PATH,
                subcmd,
                app.installation()?,
                app.platforms()?,
            )?;
        }
        Some(Cmd::Info) => print_info(app.name)?,
        Some(Cmd::Self_ { subcmd }) => self_::execute(&subcmd)?,
        None => print_info(Some(app.installation()?))?,
    };
    Ok(())
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Build project with MSBuild
    #[command(alias = "msbuild")]
    Build {
        #[command(flatten)]
        options: radstudio::msbuild::Options,
    },

    /// Build project with bds.exe
    ///
    /// Using bds.exe to build resolves the issue where the Community and Trial editions
    /// show the prompt: `This version of the product does not support command line compiling`.
    ///
    /// This command uses exactly the same command-line options as the `build` command, which uses MSBuild.
    Bds {
        #[command(flatten)]
        options: radstudio::msbuild::Options,
    },

    /// Delphi command-line compiler for Win32
    Dcc32 {
        #[command(flatten)]
        options: radstudio::dcc::Options,
    },

    /// Delphi command-line compiler for Win64
    Dcc64 {
        #[command(flatten)]
        options: radstudio::dcc::Options,
    },

    /// Delphi command-line compiler for WinARM64EC
    Dccarm64ec {
        #[command(flatten)]
        options: radstudio::dcc::Options,
    },

    /// Resource compiler (brcc32.exe)
    #[command(alias = "brcc32")]
    Brcc {
        #[command(flatten)]
        options: radstudio::brcc::Options,
    },

    /// Manage IDE environment variables
    Env {
        #[command(subcommand)]
        subcmd: Option<env::EnvCmd>,
    },

    /// Manage IDE environment variable PATH
    #[command(aliases = ["envpath", "path"])]
    EnvPath {
        #[command(subcommand)]
        subcmd: Option<paths::PathsCmd>,
    },

    /// Manage IDE library path
    #[command(aliases = ["librarypath", "lib-path", "libpath"])]
    LibraryPath {
        #[command(subcommand)]
        subcmd: Option<paths::PathsCmd>,
    },

    /// Manage IDE browsing path
    #[command(alias = "browsingpath")]
    BrowsingPath {
        #[command(subcommand)]
        subcmd: Option<paths::PathsCmd>,
    },

    /// Print installed RAD Studio product information
    Info,

    /// Manage this tool itself
    Self_ {
        #[command(subcommand)]
        subcmd: self_::SelfCmd,
    },
}

#[derive(Parser)]
#[command(name = APP_NAME, version, about = APP_NAME)]
#[command(arg_required_else_help = true, styles = clap_cargo::style::CLAP_STYLING)]
struct App {
    /// Specify the RAD Studio name or version (e.g. 13, XE2, or Florence)
    ///
    /// Supported values:
    /// - Product name: "RAD Studio 13" or "RAD Studio 13.1"
    /// - Product codename: Florence, Rio, Berlin
    /// - Product version: 13, 12, XE2, XE8
    ///
    /// If omitted, the latest installed version is used.
    #[arg(verbatim_doc_comment, value_parser = parse_name)]
    name: Option<&'static Installation>,

    #[command(subcommand)]
    subcmd: Option<Cmd>,

    #[command(flatten, next_help_heading = "Global Options")]
    global: GlobalOptions,
}

#[derive(Debug, clap::Args)]
struct GlobalOptions {
    /// Specify the toolchain or IDE architecture
    #[arg(
        short,
        long,
        alias = "arch",
        value_name = "ARCH",
        value_enum,
        ignore_case = true,
        global = true,
        display_order = 1
    )]
    architecture: Option<Architecture>,
    /// Specify the target platform
    #[arg(
        short,
        long,
        value_enum,
        ignore_case = true,
        global = true,
        display_order = 2
    )]
    platform: Option<Platform>,
}

impl App {
    fn installation(&self) -> anyhow::Result<&'static Installation> {
        match self.name {
            Some(i) => Ok(i),
            None => latest_installation(),
        }
    }

    fn ide_architectures(&self) -> anyhow::Result<Architectures> {
        let ide_archs = self.installation()?.product_info().ide_architectures();
        Ok(match &self.global.architecture {
            Some(a) if ide_archs.contains(a) => std::iter::once(a.to_owned()).collect(),
            Some(a) => bail!("{} is not installed", a.ide_name()),
            None => ide_archs,
        })
    }

    fn platforms(&self) -> anyhow::Result<Platforms> {
        let platforms = self.installation()?.product_info().platforms();
        Ok(match &self.global.platform {
            Some(p) if platforms.contains(p) => std::iter::once(p.to_owned()).collect(),
            Some(p) => bail!("{p} platform is not installed"),
            None => platforms,
        })
    }

    fn dcc_execute(
        &self,
        clt: &CommandLineTool,
        options: &radstudio::dcc::Options,
    ) -> anyhow::Result<()> {
        self.installation()?
            .dcc(clt, &self.global.architecture)
            .context(err_clt_not_found(clt))?
            .execute(options)?;
        Ok(())
    }
}

fn latest_installation() -> anyhow::Result<&'static Installation> {
    installations()
        .latest()
        .context("no RAD Studio installations found")
}

fn parse_name(name: &str) -> Result<&'static Installation, String> {
    installations()
        .find_by_name(name)
        .ok_or("no installed RAD Studio matched".to_string())
}

fn err_clt_not_found(clt: &CommandLineTool) -> String {
    format!("{} not found", clt.file_name())
}

fn print_installation(installation: &Installation, id: Option<usize>) {
    id.inspect(|id| print!("{id}. "));
    println!("{}", installation.product_info().display_name());
    println!("{}", installation.product_info());
}

fn print_info(installation: Option<&Installation>) -> anyhow::Result<()> {
    match installation {
        Some(i) => print_installation(i, None),
        None => {
            if installations().count() <= 1 {
                print_installation(latest_installation()?, None);
            } else {
                for (id, i) in installations().iter().enumerate() {
                    print_installation(i, Some(id + 1));
                }
            }
        }
    }
    Ok(())
}
