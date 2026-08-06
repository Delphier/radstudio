mod selfcmd;

use anyhow::Context;
use clap::Parser;
use clap::Subcommand;
use std::sync::OnceLock;

const APP_NAME: &'static str = "RAD Studio CLI";

static INSTALLATIONS: OnceLock<radstudio::Installations> = OnceLock::new();
fn installations() -> &'static radstudio::Installations {
    INSTALLATIONS.get_or_init(|| radstudio::find().unwrap())
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    let installation = match app.name {
        Some(i) => i,
        None => installations()
            .latest()
            .context("no RAD Studio installations found")?,
    };
    match app.subcmd {
        Some(cmd) => match cmd {
            Cmd::Info => print_info(app.name),
            Cmd::Self_ { subcmd } => selfcmd::execute(subcmd)?,
        },
        None => print_info(Some(installation)),
    };
    Ok(())
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
    #[arg(verbatim_doc_comment, value_parser = parse_name)]
    name: Option<&'static radstudio::Installation>,
    #[command(subcommand)]
    subcmd: Option<Cmd>,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Print installed RAD Studio product information
    Info,
    /// Manage this tool itself
    Self_ {
        #[command(subcommand)]
        subcmd: selfcmd::SelfCmd,
    },
}

fn print_info(installation: Option<&'static radstudio::Installation>) {
    match installation {
        Some(i) => {
            println!("{}", i.product_info().full_name());
            println!("{}", i.product_info());
        }
        None => {
            for (index, install) in installations().iter().enumerate() {
                println!("{}. {}", index + 1, install.product_info().full_name());
                println!("{}", install.product_info());
            }
        }
    }
}

fn parse_name(name: &str) -> Result<&'static radstudio::Installation, String> {
    installations()
        .find_by_name(name)
        .ok_or("no installed RAD Studio matched".to_string())
}
