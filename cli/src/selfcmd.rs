use anyhow::Context;
use clap::Subcommand;
use std::{
    env::{current_exe, join_paths, split_paths},
    path::{Path, PathBuf},
};
use windows_registry::{CURRENT_USER, Key};

const ENVIRONMENT: &'static str = "Environment";
const RAD_STUDIO_CLI: &'static str = "RADStudioCLI";

#[derive(Debug, Subcommand)]
pub(crate) enum SelfCmd {
    /// Add this tool to the PATH environment variable
    Install,
    /// Remove this tool from the PATH environment variable
    Uninstall,
}

fn env_var_placeholder(name: &'static str) -> String {
    format!("%{name}%")
}

fn env_var_path(key: &Key) -> Vec<PathBuf> {
    split_paths(&key.get_string("Path").unwrap_or_default()).collect()
}

fn set_env_var_path(key: &Key, paths: Vec<PathBuf>) -> anyhow::Result<()> {
    key.set_expand_string(
        "Path",
        join_paths(paths)?
            .to_str()
            .context("failed to convert joined PATH to a valid UTF-8 string")?,
    )?;
    Ok(())
}

fn same_path(p: &Path, s: &str) -> bool {
    p.to_str().unwrap_or_default().eq_ignore_ascii_case(s)
}

pub(crate) fn execute(cmd: SelfCmd) -> anyhow::Result<()> {
    let key = CURRENT_USER.create(ENVIRONMENT)?;
    let mut paths = env_var_path(&key);
    let placeholder = env_var_placeholder(RAD_STUDIO_CLI);
    match cmd {
        SelfCmd::Install => {
            key.set_string(
                RAD_STUDIO_CLI,
                current_exe()?
                    .parent()
                    .context("current executable path has no parent directory")?
                    .to_str()
                    .context("executable directory path is not valid UTF-8")?,
            )?;
            if !paths.iter().any(|p| same_path(p, &placeholder)) {
                paths.push(PathBuf::from(placeholder));
                set_env_var_path(&key, paths)?;
            };
            println!("✔️ {} installed successfully", crate::APP_NAME);
            println!("➡️ Added to PATH - open a new terminal for it to take effect.");
        }
        SelfCmd::Uninstall => {
            paths.retain(|p| !same_path(p, &placeholder));
            set_env_var_path(&key, paths)?;
            let _ = key.remove_value(RAD_STUDIO_CLI);
            println!("✔️ {} uninstalled\n➡️ Removed from PATH.", crate::APP_NAME);
        }
    }
    Ok(())
}
