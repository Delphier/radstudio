use crate::{Architecture, Platform};
use std::process::{Command, ExitStatus};
use std::{collections::BTreeMap, fmt::Display, os::windows::process::CommandExt, path::PathBuf};
use unicase::Ascii;

pub mod patcher;

mod consts {
    pub const EXT_GROUPPROJ: &'static str = "groupproj";
    pub const EXT_DPROJ: &'static str = "dproj";
    pub const EXT_CBPROJ: &'static str = "cbproj";

    pub const PROPERTY_GROUP: &'static str = "PropertyGroup";

    pub const CONFIG: &'static str = "Config";
    pub const PLATFORM: &'static str = "Platform";
    pub const VERINFO_INCLUDE_VERINFO: &'static str = "VerInfo_IncludeVerInfo";
    pub const VERINFO_KEYS: &'static str = "VerInfo_Keys";
    pub const DCC_USE_MSBUILD_EXTERNALLY: &'static str = "DCC_UseMSBuildExternally";
    pub const DCC_PREFERRED_TOOL_ARCHITECTURE: &'static str = "DCC_PreferredToolArchitecture";
    pub const PRE_BUILD_EVENT: &'static str = "PreBuildEvent";
}
use consts::*;

#[derive(Debug, clap::Args)]
pub struct Options {
    /// Specify project file path
    pub file: PathBuf,
    /// Specify the configuration name (e.g., Debug or Release).
    ///
    /// If omitted, the default configuration defined in the project file is used.
    #[arg(short, long)]
    pub config: Option<String>,
    #[command(flatten)]
    pub version_info: VersionInfo,
}

#[derive(Debug, Clone, clap::Args, serde::Serialize)]
#[command(
    rename_all = "PascalCase",
    next_help_heading = "Include Version Information"
)]
#[serde(rename_all = "PascalCase")]
pub struct VersionInfo {
    /// Company that produced the file — for example, Microsoft Corporation
    #[arg(long)]
    company_name: Option<String>,
    /// File description to be presented to users
    #[arg(long)]
    file_description: Option<String>,
    /// Version number of the file — for example, 3.10 or 5.00.RC2
    #[arg(long)]
    file_version: Option<String>,
    /// Internal name of the file, if one exists
    #[arg(long)]
    internal_name: Option<String>,
    /// Copyright notices that apply to the file
    #[arg(long)]
    legal_copyright: Option<String>,
    /// Trademarks and registered trademarks that apply to the file
    #[arg(long)]
    legal_trademarks: Option<String>,
    /// Original name of the file, not including a path
    #[arg(long)]
    original_filename: Option<String>,
    /// Name of the product with which the file is distributed
    #[arg(long)]
    product_name: Option<String>,
    /// Version of the product with which the file is distributed — for example, 3.10 or 5.00.RC2
    #[arg(long)]
    product_version: Option<String>,
    /// Additional information that should be displayed for diagnostic purposes
    #[arg(long)]
    comments: Option<String>,
    /// Specify version information in the format "key1=value1;key2=value2;..."
    ///
    /// Values specified by individual options with the same names override those specified here.
    #[arg(long = "VersionInfo")]
    #[serde(skip)]
    raw: Option<String>,
}

impl Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = BTreeMap::new();
        if let Ok(serde_json::Value::Object(obj)) = serde_json::to_value(self) {
            for (k, v) in obj {
                if let serde_json::Value::String(s) = v {
                    map.insert(Ascii::new(k), s);
                }
            }
        };
        if let Some(raw) = &self.raw {
            for (k, v) in raw.split(';').filter_map(|s| s.split_once('=')) {
                map.entry(Ascii::new(k.trim().to_owned()))
                    .or_insert(v.trim().to_owned());
            }
        }

        for (i, (k, v)) in map.iter().enumerate() {
            if i > 0 {
                write!(f, ";")?;
            }
            write!(f, "{k}={v}")?;
        }
        Ok(())
    }
}

pub struct MsBuild {
    rsvars_bat: PathBuf,
}

impl MsBuild {
    pub(crate) fn new(rsvars_bat: PathBuf) -> Self {
        Self { rsvars_bat }
    }

    pub fn execute(
        &self,
        arch: &Option<Architecture>,
        platform: &Option<Platform>,
        options: &Options,
    ) -> std::io::Result<ExitStatus> {
        let mut args = vec![
            format!("\"{}\"", options.file.display()),
            "/t:Build".to_string(),
        ];
        if let Some(config) = &options.config {
            args.push(format!("/p:{CONFIG}={config}"));
        }
        if let Some(platform) = platform {
            args.push(format!("/p:{PLATFORM}={platform}"));
        }
        let version_info = options.version_info.to_string();
        if !version_info.is_empty() {
            args.push(format!("/p:{VERINFO_INCLUDE_VERINFO}=true"));
            args.push(format!("/p:{VERINFO_KEYS}=\"{version_info}\""));
        }
        if let Some(arch) = arch {
            args.push(format!("/p:{DCC_PREFERRED_TOOL_ARCHITECTURE}={arch}"));
        }
        let cmd_arg = format!(
            "\"{}\" && MSBuild.exe {}",
            self.rsvars_bat.display(),
            args.join(" ")
        );
        Command::new("cmd.exe")
            .arg("/C")
            .raw_arg(format!("\" {cmd_arg} \""))
            .status()
    }
}
