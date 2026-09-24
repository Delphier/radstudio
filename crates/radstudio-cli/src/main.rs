mod env;
mod info;
mod package;
mod paths;
mod self_;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use radstudio::{
    Architecture, Architectures, CommandLineTool, Installation, Installations, Platform, Platforms,
    msbuild::{self, Execute},
};
use std::{
    process::ExitStatus,
    sync::{LazyLock, OnceLock},
};

const APP_NAME: &str = "RAD Studio CLI";

static INSTALLATIONS: LazyLock<Installations> = LazyLock::new(|| {
    let installations = radstudio::find().unwrap();
    if installations.is_empty() {
        panic!("No RAD Studio installations found")
    }
    installations
});

static LATEST_INSTALLATION: LazyLock<&Installation> =
    LazyLock::new(|| &INSTALLATIONS[INSTALLATIONS.len() - 1]);

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    match &app.subcmd {
        Some(Cmd::Build { options }) => {
            app.build_execute(
                app.installation()
                    .product_info()
                    .supports_command_line_compilation()
                    || app.global.no_bds,
                options,
            )?;
        }
        Some(Cmd::Bds { options }) => {
            app.build_execute(false, options)?;
        }
        Some(Cmd::Dcc { options }) => match &app.global.platform {
            Some(p) => app.dcc_execute(&p.dcc(), options)?,
            None => bail!("the platform must be specified"),
        },
        Some(Cmd::Dcc32 { options }) => {
            app.dcc_execute(&CommandLineTool::DCC32, options)?;
        }
        Some(Cmd::Dcc64 { options }) => {
            app.dcc_execute(&CommandLineTool::DCC64, options)?;
        }
        Some(Cmd::Dccarm64ec { options }) => {
            app.dcc_execute(&CommandLineTool::DCCARM64EC, options)?;
        }
        Some(Cmd::Package { subcmd }) => {
            subcmd.execute(&app)?;
        }
        Some(Cmd::Brcc { options }) => {
            app.installation()
                .brcc32(&app.global.architecture)
                .context(err_clt_not_found(&CommandLineTool::BRCC32))?
                .execute(options)?;
        }
        Some(Cmd::Env { subcmd }) => {
            env::EnvCmd::execute(subcmd, app.installation(), app.ide_architectures()?)?;
        }
        Some(Cmd::EnvPath { subcmd }) => {
            env::path::execute(subcmd, app.installation(), app.ide_architectures()?)?;
        }
        Some(Cmd::LibraryPath { subcmd }) => {
            paths::PathsCmd::execute(
                "Library Path",
                Installation::LIBRARY_PATH,
                subcmd,
                app.installation(),
                app.platforms()?,
            )?;
        }
        Some(Cmd::BrowsingPath { subcmd }) => {
            paths::PathsCmd::execute(
                "Browsing Path",
                Installation::BROWSING_PATH,
                subcmd,
                app.installation(),
                app.platforms()?,
            )?;
        }
        Some(Cmd::Select { message, multi }) => {
            let installations = INSTALLATIONS.select(message, *multi);
            let output = if app.global.json {
                let data = installations
                    .iter()
                    .map(|i| i.product_info().data())
                    .collect::<Result<Vec<_>, _>>()?;
                serde_json::to_string_pretty(&data)?
            } else {
                installations
                    .iter()
                    .map(|i| format!(r#""{}""#, i.product_info().name()))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            print!("{output}");
        }
        Some(Cmd::Info) => info::print(app.name, app.global.json)?,
        Some(Cmd::Self_ { subcmd }) => self_::execute(&subcmd)?,
        None => info::print(Some(app.installation()), app.global.json)?,
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

    /// Delphi command-line compiler
    Dcc {
        #[command(flatten)]
        options: radstudio::dcc::Options,
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

    /// Manage design-time packages
    Package {
        #[command(subcommand)]
        subcmd: package::PackageCmd,
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

    /// Select IDE interactively
    Select {
        /// Prompt message
        #[arg(default_value = "Select IDE:")]
        message: String,
        /// Allow multi-select
        #[arg(short, long)]
        multi: bool,
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

    #[arg(skip)]
    installation: OnceLock<&'static Installation>,

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
        aliases = ["arch", "ide"],
        value_name = "ARCH",
        ignore_case = true,
        global = true,
        display_order = 1
    )]
    architecture: Option<Architecture>,

    /// Specify the target platform
    #[arg(short, long, ignore_case = true, global = true, display_order = 2)]
    platform: Option<Platform>,

    /// Output JSON format data
    #[arg(long, global = true)]
    json: bool,

    /// No fallback to bds.exe when command-line compilation is unsupported
    #[arg(long, alias = "nobds", global = true)]
    no_bds: bool,

    /// No splash screen or no logo
    #[arg(long, aliases = ["nosplash", "no-logo", "nologo", "ns"], global = true)]
    no_splash: bool,
}

impl App {
    fn installation(&self) -> &'static Installation {
        *self.installation.get_or_init(|| match self.name {
            Some(i) => i,
            None => &LATEST_INSTALLATION,
        })
    }

    fn ide_architecture(&self) -> anyhow::Result<Architecture> {
        let mut ide_archs = self.installation().product_info().ide_architectures();
        Ok(match &self.global.architecture {
            Some(a) if ide_archs.contains(a) => a.to_owned(),
            Some(a) => bail!(err_ide_not_installed(a)),
            None if ide_archs.len() == 1 => ide_archs.pop_first().unwrap(),
            None => bail!("IDE architecture must be specified"),
        })
    }

    fn ide_architectures(&self) -> anyhow::Result<Architectures> {
        let ide_archs = self.installation().product_info().ide_architectures();
        Ok(match &self.global.platform {
            Some(p) if self.global.architecture.is_none() => {
                let arch = ide_archs
                    .into_iter()
                    .find(|a| &a.platform() == p)
                    .context(format!("{p} does not map to a valid IDE architecture"))?;
                Architectures::from([arch])
            }
            _ => match &self.global.architecture {
                Some(a) if ide_archs.contains(a) => std::iter::once(a.to_owned()).collect(),
                Some(a) => bail!(err_ide_not_installed(a)),
                None => ide_archs,
            },
        })
    }

    fn platforms(&self) -> anyhow::Result<Platforms> {
        let platforms = self.installation().product_info().platforms();
        Ok(match &self.global.platform {
            Some(p) if platforms.contains(p) => std::iter::once(p.to_owned()).collect(),
            Some(p) => bail!("{p} platform is not installed"),
            None => platforms,
        })
    }

    fn bds(&self) -> anyhow::Result<radstudio::bds::Bds> {
        self.installation()
            .bds(&self.global.architecture)
            .context(format!(
                "bds.exe not found{}",
                self.global
                    .architecture
                    .as_ref()
                    .map(|a| format!(" ({} not installed)", a.ide_name()))
                    .unwrap_or_default()
            ))
    }

    fn build_execute(
        &self,
        is_msbuild: bool,
        options: &msbuild::Options,
    ) -> anyhow::Result<ExitStatus> {
        let mut options = options.clone();
        options.no_logo = self.global.no_splash;

        let exe: &dyn Execute = if is_msbuild {
            &self
                .installation()
                .msbuild(&self.global.architecture)
                .context("MSBuild.exe not found")?
        } else {
            if !self
                .installation()
                .product_info()
                .supports_command_line_compilation()
            {
                options.preferred_tool_architecture = None;
            };
            &self.bds()?
        };
        let status = exe.execute(&self.global.platform, &options)?;
        Ok(status)
    }

    fn dcc_execute(
        &self,
        clt: &CommandLineTool,
        options: &radstudio::dcc::Options,
    ) -> anyhow::Result<()> {
        if self
            .installation()
            .product_info()
            .supports_command_line_compilation()
            || self.global.no_bds
        {
            self.installation()
                .dcc(clt, &self.global.architecture)
                .context(err_clt_not_found(clt))?
                .execute(options)?;
            return Ok(());
        };

        let curdir = &std::env::current_dir()?;
        let file = msbuild::FileInfo::new(&options.file, Some(curdir));
        let temp = msbuild::generator_project_file(&file, &clt.platform(), options)?;
        let input = match &temp {
            Some(t) => &t.0,
            None => &options.file,
        };
        self.bds()?.build(input, self.global.no_splash)?;
        Ok(())
    }
}

fn parse_name(name: &str) -> Result<&'static Installation, String> {
    INSTALLATIONS
        .find_by_name(name)
        .ok_or("no installed RAD Studio matched".to_string())
}

fn err_ide_not_installed(arch: &Architecture) -> String {
    format!("{} is not installed", arch.ide_name())
}

fn err_clt_not_found(clt: &CommandLineTool) -> String {
    format!("{} not found", clt.file_name())
}
