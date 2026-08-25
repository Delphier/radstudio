use anyhow::Context;
use clap::Subcommand;
use std::env::current_exe;

const RAD_STUDIO_CLI: &'static str = "RADStudioCLI";

#[derive(Debug, Subcommand)]
pub(crate) enum SelfCmd {
    /// Add this tool to the PATH environment variable
    Install,
    /// Remove this tool from the PATH environment variable
    Uninstall,
}

pub(crate) fn execute(cmd: &SelfCmd) -> anyhow::Result<()> {
    let envz = envz::Environment::new()?;
    let placeholder = envz::var::placeholder(RAD_STUDIO_CLI);
    match cmd {
        SelfCmd::Install => {
            envz.set(
                RAD_STUDIO_CLI,
                current_exe()?
                    .parent()
                    .context("current executable path has no parent directory")?,
            )?;
            envz.path_push(placeholder)?;
            println!("✔️ {} installed successfully", crate::APP_NAME);
            println!("➡️ Added to PATH - open a new terminal for it to take effect.");
        }
        SelfCmd::Uninstall => {
            envz.path_remove(placeholder)?;
            envz.remove(RAD_STUDIO_CLI)?;
            println!("✔️ {} uninstalled\n➡️ Removed from PATH.", crate::APP_NAME);
        }
    }
    Ok(())
}
