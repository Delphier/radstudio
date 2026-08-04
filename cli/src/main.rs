mod selfcmd;

use clap::Parser;
use clap::Subcommand;

pub const APP_NAME: &'static str = "RAD Studio CLI";

fn main() -> anyhow::Result<()> {
    let app = App::parse();
    if let Some(cmd) = app.subcmd {
        match cmd {
            Cmd::Info => {
                let installs = radstudio::find().unwrap();
                for (index, install) in installs.iter().enumerate() {
                    println!("{}. {}", index + 1, install.product_info().full_name());
                    println!("{}", install.product_info());
                }
            }
            Cmd::Self_ { subcmd } => selfcmd::execute(subcmd)?,
        }
    };
    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = APP_NAME, version, about)]
#[command(styles = clap_cargo::style::CLAP_STYLING)]
struct App {
    /// Specify the RAD Studio name or version (e.g. 13, XE2, or Florence)
    name: Option<String>,
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
