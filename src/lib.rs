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

pub enum Platform {
    Win32,
    Win64,
    Win64x,
    WinARM64EC,
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
        self.globals.get("RootDir").cloned().unwrap_or_default().into()
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
}

impl Display for ProductInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table
            .load_preset(comfy_table::presets::UTF8_FULL_CONDENSED)
            .set_header(vec!["Property", "Type", "Value"])
            .add_row(vec!["Known", "Boolean", &self.is_known().to_string()])
            .add_row(vec!["Version", "String", self.version()])
            .add_row(vec!["Version Number", "UInt", &self.version_number().to_string()])
            .add_row(vec!["Product Family", "String", self.product_family()])
            .add_row(vec!["Product Version", "String", &self.product_version()])
            .add_row(vec!["Product Name", "String", &self.product_name()])
            .add_row(vec!["Update Number", "UInt", &self.update_number().to_string()])
            .add_row(vec!["Name", "String", &self.name()])
            .add_row(vec!["Code Name", "String", self.code_name()])
            .add_row(vec!["Full Name", "String", &self.full_name()])
            .add_row(vec!["Root Dir", "String", &self.root_dir().display().to_string()])
            .add_row(vec!["Personalities", "Set", &format!("{:?}", self.personalities())])
            .add_row(vec!["Architectures", "Set", &format!("{:?}", self.architectures())]);

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

type Installations = Vec<Installation>;

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
    Ok(installs)
}
