pub mod consts;

use crate::consts::VersionInfo;
use comfy_table::Table;
use std::{
    collections::{BTreeSet, HashMap},
    fmt::Display,
    path::PathBuf,
    str::FromStr,
};
use strum::IntoEnumIterator;
use windows_registry::{CURRENT_USER, Key, Result};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, strum::EnumString)]
pub enum Personality {
    #[strum(serialize = "Delphi.Win32", to_string = "Delphi")]
    Delphi,
    #[strum(serialize = "BCB", to_string = "C++Builder")]
    CBuilder,
}

pub type Personalities = BTreeSet<Personality>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::EnumIter)]
pub enum Architecture {
    IntelX86,
    IntelX64,
}

impl Architecture {
    fn bin_dir_name(&self) -> &'static str {
        match self {
            Architecture::IntelX86 => "bin",
            Architecture::IntelX64 => "bin64",
        }
    }

    fn reg_name_suffix(&self) -> &'static str {
        match self {
            Architecture::IntelX86 => "",
            Architecture::IntelX64 => " x64",
        }
    }
}

pub type Architectures = BTreeSet<Architecture>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, strum::EnumIter)]
pub enum CommandLineTool {
    DCC32,
    DCC64,
    BCC64X,
    DCCARM64EC,
}

impl CommandLineTool {
    fn exe_path(&self, product_info: &ProductInfo, arch: &Architecture) -> PathBuf {
        product_info.bin_dir(arch).join(format!("{self:?}.exe"))
    }
}

pub type CommandLineTools = BTreeSet<CommandLineTool>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, strum::EnumIter)]
pub enum Platform {
    Win32,
    Win64,
    Win64x,
    WinARM64EC,
}

impl Platform {
    fn command_line_tool(&self) -> CommandLineTool {
        match self {
            Platform::Win32 => CommandLineTool::DCC32,
            Platform::Win64 => CommandLineTool::DCC64,
            Platform::Win64x => CommandLineTool::BCC64X,
            Platform::WinARM64EC => CommandLineTool::DCCARM64EC,
        }
    }
}

pub type Platforms = BTreeSet<Platform>;

#[derive(Debug)]
pub struct ProductInfo {
    globals: HashMap<String, String>,
    version_info: Option<&'static VersionInfo>,
    version: String,
    product_name: String,
    personalities: Personalities,
    update_number: u32,
}

impl ProductInfo {
    fn new(root_key: &Key, version: String) -> Result<Self> {
        let product_name;
        let personalities;
        if let Ok(key) = root_key.open("Personalities") {
            product_name = key.get_string("").unwrap_or_default();
            personalities = key
                .values()?
                .filter_map(|(n, _)| Personality::from_str(&n).ok())
                .collect()
        } else {
            product_name = String::new();
            personalities = Personalities::new();
        };

        Ok(Self {
            globals: root_key
                .values()?
                .map(|(n, v)| (n, v.try_into().unwrap_or_default()))
                .collect(),
            version_info: VersionInfo::new(&version),
            version,
            product_name,
            personalities,
            update_number: if let Ok(key) = root_key.open("InstalledUpdates") {
                key.get_string("Main Product Update")
                    .unwrap_or_default()
                    .split("Update")
                    .nth(1)
                    .unwrap_or_default()
                    .trim()
                    .parse()
                    .unwrap_or_default()
            } else {
                0
            },
        })
    }

    pub fn is_known(&self) -> bool {
        self.version_info.is_some()
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn version_number(&self) -> u32 {
        self.version().parse::<f64>().unwrap_or_default() as u32
    }

    pub fn compiler_version(&self) -> String {
        match self.version_number() {
            n @ (2..=4 | 6..=12) => format!("{}.0", n + 14),
            5 => "18.5".to_string(),
            n @ 14..=23 => format!("{}.0", n + 13),
            37.. => self.version().to_string(),
            _ => "".to_string(),
        }
    }

    pub fn compiler_version_number(&self) -> u32 {
        (self.compiler_version().parse::<f64>().unwrap_or_default() * 10.0) as u32
    }

    pub fn package_version(&self) -> String {
        match self.version_number() {
            n @ (2..=6 | 14..=23) => (n + 6).to_string(),
            n @ 7..=12 => (n + 7).to_string(),
            n @ (37..) => n.to_string(),
            _ => "".to_string(),
        }
    }

    pub fn package_version_number(&self) -> u32 {
        self.package_version().parse::<u32>().unwrap_or_default() * 10
    }

    pub fn product_family(&self) -> &'static str {
        match self.version_number() {
            ..=3 => "Delphi",
            4 => "Borland Developer Studio",
            _ => "RAD Studio",
        }
    }

    pub fn product_version(&self) -> String {
        match self.version_info {
            Some(v) => v.product_version.to_string(),
            None => format!("<{}>", self.version()),
        }
    }

    pub fn product_name(&self) -> String {
        if self.is_known() || self.product_name.is_empty() {
            return format!("{} {}", self.product_family(), self.product_version());
        }
        self.product_name.clone()
    }

    pub fn update_number(&self) -> &u32 {
        &self.update_number
    }

    pub fn name(&self) -> String {
        match self.update_number() {
            0 => self.product_name(),
            _ => format!("{}.{}", self.product_name(), self.update_number()),
        }
    }

    pub fn code_name(&self) -> &'static str {
        match self.version_info {
            Some(v) => v.code_name,
            None => "",
        }
    }

    pub fn full_name(&self) -> String {
        if self.code_name().is_empty() {
            self.name()
        } else {
            format!("{} {}", self.name(), self.code_name())
        }
    }

    pub fn root_dir(&self) -> PathBuf {
        self.globals
            .get("RootDir")
            .cloned()
            .unwrap_or_default()
            .into()
    }

    pub fn personalities(&self) -> &Personalities {
        &self.personalities
    }

    pub fn architectures(&self) -> Architectures {
        Architecture::iter()
            .filter_map(|a| self.app(&a).exists().then_some(a))
            .collect()
    }

    pub fn bin_dir(&self, arch: &Architecture) -> PathBuf {
        self.root_dir().join(arch.bin_dir_name())
    }

    pub fn app(&self, arch: &Architecture) -> PathBuf {
        self.globals
            .get(&format!("App{}", arch.reg_name_suffix()))
            .cloned()
            .unwrap_or_default()
            .into()
    }

    pub fn command_line_tools(&self, arch: &Architecture) -> CommandLineTools {
        CommandLineTool::iter()
            .filter_map(|c| c.exe_path(self, arch).exists().then_some(c))
            .collect()
    }

    pub fn platforms(self, arch: &Architecture) -> Platforms {
        let command_line_tools = self.command_line_tools(arch);
        Platform::iter()
            .filter_map(|p| {
                command_line_tools
                    .contains(&p.command_line_tool())
                    .then_some(p)
            })
            .collect()
    }
}

impl Display for ProductInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table
            .load_preset(comfy_table::presets::UTF8_FULL_CONDENSED)
            .set_header(vec!["Property", "Type", "Value"])
            .add_row(vec!["Known", "Boolean", &self.is_known().to_string()])
            .add_row(vec!["Version", "String", self.version()])
            .add_row(vec![
                "Version Number",
                "UInt",
                &self.version_number().to_string(),
            ])
            .add_row(vec!["Compiler Version", "String", &self.compiler_version()])
            .add_row(vec![
                "Compiler Version Number",
                "UInt",
                &self.compiler_version_number().to_string(),
            ])
            .add_row(vec!["Package Version", "String", &self.package_version()])
            .add_row(vec![
                "Package Version Number",
                "UInt",
                &self.package_version_number().to_string(),
            ])
            .add_row(vec!["Product Family", "String", self.product_family()])
            .add_row(vec!["Product Version", "String", &self.product_version()])
            .add_row(vec!["Product Name", "String", &self.product_name()])
            .add_row(vec![
                "Update Number",
                "UInt",
                &self.update_number().to_string(),
            ])
            .add_row(vec!["Name", "String", &self.name()])
            .add_row(vec!["Code Name", "String", self.code_name()])
            .add_row(vec!["Full Name", "String", &self.full_name()])
            .add_row(vec![
                "Root Dir",
                "String",
                &self.root_dir().display().to_string(),
            ])
            .add_row(vec![
                "Personalities",
                "Set",
                &format!("{:?}", self.personalities()),
            ])
            .add_row(vec![
                "Architectures",
                "Set",
                &format!("{:?}", self.architectures()),
            ]);

        for arch in self.architectures() {
            let arch_name = format!("{:?}: ", arch);
            table
                .add_row(vec![
                    &format!("{arch_name}Bin Dir"),
                    "Sring",
                    &self.bin_dir(&arch).display().to_string(),
                ])
                .add_row(vec![
                    &format!("{arch_name}App"),
                    "Sring",
                    &self.app(&arch).display().to_string(),
                ])
                .add_row(vec![
                    &format!("{arch_name}Command Line Tools"),
                    "Set",
                    &format!("{:?}", self.command_line_tools(&arch)),
                ]);
        }
        writeln!(f, "{table}")
    }
}

#[derive(Debug)]
pub struct Installation {
    product_info: ProductInfo,
    //root_key: Key,
}

impl Installation {
    fn new(root_key: Key, version: String) -> Result<Self> {
        Ok(Self {
            product_info: ProductInfo::new(&root_key, version)?,
            //root_key,
        })
    }

    pub fn product_info(&self) -> &ProductInfo {
        &self.product_info
    }
}

pub struct Installations {
    items: Vec<Installation>,
}

impl Installations {
    fn new() -> Self {
        Self { items: vec![] }
    }

    fn push(&mut self, value: Installation) {
        self.items.push(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Installation> {
        self.items.iter()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Installation> {
        self.items.iter().find(|i| {
            i.product_info.product_name().eq_ignore_ascii_case(name)
                || i.product_info().name().eq_ignore_ascii_case(name)
                || i.product_info().code_name().eq_ignore_ascii_case(name)
                || i.product_info().full_name().eq_ignore_ascii_case(name)
                || i.product_info()
                    .product_version()
                    .eq_ignore_ascii_case(name)
                || i.product_info()
                    .name()
                    .replace(i.product_info().product_family(), "")
                    .trim()
                    .eq_ignore_ascii_case(name)
        })
    }

    pub fn latest(&self) -> Option<&Installation> {
        self.items.last()
    }
}

pub fn find() -> Result<Installations> {
    let mut installs = Installations::new();
    #[cfg(windows)]
    {
        for path in [
            "Software\\Borland\\BDS",
            "Software\\CodeGear\\BDS",
            "Software\\Embarcadero\\BDS",
        ] {
            let Ok(bds) = CURRENT_USER.open(path) else {
                continue;
            };
            for version in bds.keys()? {
                installs.push(Installation::new(bds.open(&version)?, version)?);
            }
        }
    }
    installs.items.sort_by(|a, b| {
        a.product_info()
            .version_number()
            .cmp(&b.product_info().version_number())
    });
    Ok(installs)
}
