mod selfcmd;

use anyhow::Context;
use clap::{Parser, Subcommand};
use radstudio::{Architecture, Installation, Installations, Platform};
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
                .msbuild(&app.architecture, &app.platform, &options)?;
        }
        Some(Cmd::Info) => print_info(app.name)?,
        Some(Cmd::Self_ { subcmd }) => selfcmd::execute(&subcmd)?,
        None => print_info(Some(app.installation()?))?,
    };
    Ok(())
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Build project file (*.dproj, *.cbproj, *.groupproj)
    #[command(alias = "msbuild")]
    Build {
        #[command(flatten)]
        options: radstudio::msbuild::Options,
    },
    /// Print installed RAD Studio product information
    Info,
    /// Manage this tool itself
    Self_ {
        #[command(subcommand)]
        subcmd: selfcmd::SelfCmd,
    },
}

#[derive(Debug, Parser)]
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
    #[command(subcommand)]
    subcmd: Option<Cmd>,
}

impl App {
    fn installation(&self) -> anyhow::Result<&'static Installation> {
        match self.name {
            Some(i) => Ok(i),
            None => latest_installation(),
        }
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

fn print_installation(installation: &Installation, id: Option<usize>) {
    if let Some(id) = id {
        print!("{}. ", id);
    }
    println!("{}", installation.product_info().full_name());
    println!("{}", installation.product_info());
}

fn print_info(installation: Option<&Installation>) -> anyhow::Result<()> {
    match installation {
        Some(i) => print_installation(i, None),
        None => match installations().count() {
            0 | 1 => print_installation(latest_installation()?, None),
            _ => {
                for (id, i) in installations().iter().enumerate() {
                    print_installation(i, Some(id + 1));
                }
            }
        },
    }
    Ok(())
}
