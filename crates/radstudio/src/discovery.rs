use crate::{brcc::Brcc, consts, dcc::Dcc, msbuild::MsBuild};
use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};
use envz::{Environment, registry::Node, registry::StringEntry};
use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};
use strum::{IntoEnumIterator, VariantArray};
use windows_registry::{CURRENT_USER, Key, Result};

pub struct BTreeSet<T>(std::collections::BTreeSet<T>);

impl<T> BTreeSet<T> {
    fn new() -> Self {
        Self(std::collections::BTreeSet::new())
    }
}

impl<T: Display> Display for BTreeSet<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            };
            write!(f, "{v}")?;
        }
        write!(f, "}}")
    }
}

impl<T> std::iter::IntoIterator for BTreeSet<T> {
    type Item = T;
    type IntoIter = std::collections::btree_set::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Ord> std::iter::FromIterator<T> for BTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(std::collections::BTreeSet::from_iter(iter))
    }
}

impl<T> std::ops::Deref for BTreeSet<T> {
    type Target = std::collections::BTreeSet<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, strum::EnumString)]
pub enum Edition {
    #[strum(serialize = "Starter")]
    Community,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, strum::EnumString, strum::Display)]
pub enum Personality {
    #[strum(serialize = "Delphi.Win32", to_string = "Delphi")]
    Delphi,
    #[strum(serialize = "BCB", to_string = "C++Builder")]
    CBuilder,
}

pub type Personalities = BTreeSet<Personality>;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    strum::EnumIter,
    strum::VariantArray,
    strum::Display,
    clap::ValueEnum,
)]
pub enum Architecture {
    /// x86 toolchain or 32-bit IDE
    #[value(aliases = ["IntelX86", "32bit", "32-bit"])]
    #[strum(to_string = "x86")]
    X86,
    /// x64 toolchain or 64-bit IDE
    #[value(aliases = ["IntelX64", "64bit", "64-bit"])]
    #[strum(to_string = "x64")]
    X64,
}

impl Architecture {
    fn bin_dir_name(&self) -> &'static str {
        match self {
            Architecture::X86 => "bin",
            Architecture::X64 => "bin64",
        }
    }

    fn reg_name_suffix(&self) -> &'static str {
        match self {
            Architecture::X86 => "",
            Architecture::X64 => " x64",
        }
    }

    pub fn ide_name(&self) -> &'static str {
        match self {
            Architecture::X86 => "32-bit IDE",
            Architecture::X64 => "64-bit IDE",
        }
    }
}

pub type Architectures = BTreeSet<Architecture>;

// Delphi toolchains: https://docwiki.embarcadero.com/RADStudio/en/Delphi_Toolchains
// C++Builder toolchains: https://docwiki.embarcadero.com/RADStudio/en/C++_Toolchains
// Command-Line Utilities Index: https://docwiki.embarcadero.com/RADStudio/en/Command-Line_Utilities_Index
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, strum::EnumIter, strum::Display)]
pub enum CommandLineTool {
    /// RAD Studio Delphi compiler for 32-bit Windows
    DCC32,
    /// RAD Studio Delphi compiler for 64-bit Windows
    DCC64,
    /// RAD Studio C++ compiler for 64-bit Windows (Modern)
    BCC64X,
    /// RAD Studio Delphi compiler for 64-bit Windows on ARM (Emulation Compatible)
    DCCARM64EC,
    /// RAD Studio Delphi compiler for 64-bit Intel macOS
    DCCOSX64,
    /// RAD Studio Delphi compiler for 64-bit ARM macOS
    DCCOSXARM64,
    /// RAD Studio Delphi compiler for 64-bit Intel Linux
    DCCLINUX64,
    /// RAD Studio Delphi compiler for 32-bit Android
    DCCAARM,
    /// RAD Studio Delphi compiler for 64-bit Android
    DCCAARM64,
    /// RAD Studio Delphi compiler for 64-bit iOS
    DCCIOSARM64,
    /// Resource compiler
    BRCC32,
}

impl CommandLineTool {
    pub fn file_name(&self) -> String {
        format!("{self:?}.exe")
    }

    fn path(&self, product_info: &ProductInfo, arch: &Architecture) -> PathBuf {
        product_info.bin_dir(arch).join(self.file_name())
    }

    fn which(&self, product_info: &ProductInfo, arch: &Option<Architecture>) -> Option<PathBuf> {
        match arch {
            Some(Architecture::X86) => &[Architecture::X86, Architecture::X64],
            Some(Architecture::X64) => &[Architecture::X64, Architecture::X86],
            None => Architecture::VARIANTS,
        }
        .iter()
        .map(|a| self.path(product_info, a))
        .find(|path| path.exists())
    }
}

pub type CommandLineTools = BTreeSet<CommandLineTool>;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, strum::EnumIter, strum::Display, clap::ValueEnum,
)]
#[value(rename_all = "verbatim")]
pub enum Platform {
    /// 32-bit Intel Windows
    Win32,
    /// 64-bit Intel Windows
    Win64,
    /// 64-bit Intel Windows (Modern), only available for C++ Builder
    Win64x,
    /// 64-bit Arm64EC Windows
    WinARM64EC,
    /// 64-bit Intel macOS
    OSX64,
    /// 64-bit ARM macOS
    OSXARM64,
    /// 64-bit Intel Linux
    Linux64,
    /// 32-bit Android
    Android32,
    /// 64-bit Android
    Android64,
    /// 64-bit iOS
    IOSDevice64,
}

impl Platform {
    fn command_line_tool(&self) -> CommandLineTool {
        match self {
            Platform::Win32 => CommandLineTool::DCC32,
            Platform::Win64 => CommandLineTool::DCC64,
            Platform::Win64x => CommandLineTool::BCC64X,
            Platform::WinARM64EC => CommandLineTool::DCCARM64EC,
            Platform::OSX64 => CommandLineTool::DCCOSX64,
            Platform::OSXARM64 => CommandLineTool::DCCOSXARM64,
            Platform::Linux64 => CommandLineTool::DCCLINUX64,
            Platform::Android32 => CommandLineTool::DCCAARM,
            Platform::Android64 => CommandLineTool::DCCAARM64,
            Platform::IOSDevice64 => CommandLineTool::DCCIOSARM64,
        }
    }
}

pub type Platforms = BTreeSet<Platform>;

pub struct ProductInfo {
    globals: HashMap<String, String>,
    version_info: Option<&'static consts::VersionInfo>,
    reg_parent: PathBuf,
    version: String,
    product_name: String,
    personalities: Personalities,
    update_number: u32,
}

impl ProductInfo {
    pub const REG_ROOT: &Key = CURRENT_USER;

    fn reg_key_with(parent: impl AsRef<Path>, version: impl AsRef<str>) -> Result<Key> {
        Self::REG_ROOT.open(parent.as_ref().join(version.as_ref()).display().to_string())
    }

    fn reg_key(&self) -> Result<Key> {
        Self::reg_key_with(&self.reg_parent, self.version())
    }

    fn new(reg_parent: PathBuf, version: String) -> Result<Self> {
        let reg_key = Self::reg_key_with(&reg_parent, &version)?;
        let product_name;
        let personalities;
        if let Ok(key) = reg_key.open("Personalities") {
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
            globals: reg_key
                .values()?
                .map(|(n, v)| (n, v.try_into().unwrap_or_default()))
                .collect(),
            version_info: consts::VersionInfo::new(&version),
            reg_parent,
            version,
            product_name,
            personalities,
            update_number: if let Ok(key) = reg_key.open("InstalledUpdates") {
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

    pub fn edition(&self) -> Option<Edition> {
        self.globals.get("Edition").and_then(|s| s.parse().ok())
    }

    pub fn display_name(&self) -> String {
        match self.edition() {
            Some(e) => format!("{} {e:?} Edition", self.full_name()),
            None => self.full_name(),
        }
    }

    pub fn root_dir(&self) -> PathBuf {
        self.globals
            .get("RootDir")
            .map(PathBuf::from)
            .unwrap_or_default()
    }

    pub fn personalities(&self) -> &Personalities {
        &self.personalities
    }

    pub fn architectures(&self) -> Architectures {
        Architecture::iter()
            .filter_map(|a| self.rsvars_bat(&a).exists().then_some(a))
            .collect()
    }

    pub fn ide_architectures(&self) -> Architectures {
        Architecture::iter()
            .filter_map(|a| self.bds_exe(&a).exists().then_some(a))
            .collect()
    }

    pub fn platforms(&self) -> Platforms {
        Platform::iter()
            .filter_map(|p| p.command_line_tool().which(self, &None).map(|_| p))
            .collect()
    }

    pub fn bin_dir(&self, arch: &Architecture) -> PathBuf {
        self.root_dir().join(arch.bin_dir_name())
    }

    pub fn rsvars_bat(&self, arch: &Architecture) -> PathBuf {
        self.bin_dir(arch).join(match arch {
            Architecture::X86 => "rsvars.bat",
            Architecture::X64 => "rsvars64.bat",
        })
    }

    pub fn bds_exe(&self, arch: &Architecture) -> PathBuf {
        self.globals
            .get(&format!("App{}", arch.reg_name_suffix()))
            .cloned()
            .unwrap_or_default()
            .into()
    }

    pub fn command_line_tools(&self, arch: &Architecture) -> CommandLineTools {
        CommandLineTool::iter()
            .filter_map(|c| c.path(self, arch).exists().then_some(c))
            .collect()
    }
}

impl Display for ProductInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table
            .load_style(UTF8_FULL_CONDENSED)
            .set_content_arrangement(ContentArrangement::Dynamic)
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
                "Edition",
                "Enum",
                &self
                    .edition()
                    .map_or("<Unknown>".to_string(), |e| format!("{e:?}")),
            ])
            .add_row(vec!["Display Name", "String", &self.display_name()])
            .add_row(vec![
                "Root Dir",
                "Path",
                &self.root_dir().display().to_string(),
            ])
            .add_row(vec![
                "Personalities",
                "Set",
                &format!("{}", self.personalities()),
            ])
            .add_row(vec![
                "Toolchain Architectures",
                "Set",
                &format!("{}", self.architectures()),
            ])
            .add_row(vec![
                "IDE Architectures",
                "Set",
                &format!("{}", self.ide_architectures()),
            ])
            .add_row(vec!["Platforms", "Set", &format!("{}", self.platforms())]);

        for arch in self.architectures() {
            let arch_name = format!("{arch}: ");
            table
                .add_row(vec![
                    &format!("{arch_name}Bin Dir"),
                    "Path",
                    &self.bin_dir(&arch).display().to_string(),
                ])
                .add_row(vec![
                    &format!("{arch_name}rsvars.bat"),
                    "Path",
                    &self.rsvars_bat(&arch).display().to_string(),
                ])
                .add_row(vec![
                    &format!("{arch_name}bds.exe"),
                    "Path",
                    &self.bds_exe(&arch).display().to_string(),
                ])
                .add_row(vec![
                    &format!("{arch_name}Command-Line Tools"),
                    "Set",
                    &format!("{}", self.command_line_tools(&arch)),
                ]);
        }
        writeln!(f, "{table}")
    }
}

pub struct Installation {
    product_info: ProductInfo,
}

impl Installation {
    fn new(reg_parent: PathBuf, version: String) -> Result<Self> {
        Ok(Self {
            product_info: ProductInfo::new(reg_parent, version)?,
        })
    }

    pub fn product_info(&self) -> &ProductInfo {
        &self.product_info
    }

    pub fn msbuild(&self, arch: &Option<Architecture>) -> Option<MsBuild> {
        let product_info = self.product_info();
        let archs = product_info.architectures();
        let arch_valid = match arch.as_ref() {
            Some(a) if archs.contains(a) => a,
            Some(_) => return None,
            None => archs.first()?,
        };
        Some(MsBuild::new(
            product_info.rsvars_bat(arch_valid),
            arch.clone(),
        ))
    }

    pub fn dcc(&self, clt: &CommandLineTool, arch: &Option<Architecture>) -> Option<Dcc> {
        clt.which(self.product_info(), arch)
            .map(|path| Dcc::new(path))
    }

    pub fn dcc32(&self, arch: &Option<Architecture>) -> Option<Dcc> {
        self.dcc(&CommandLineTool::DCC32, arch)
    }

    pub fn dcc64(&self, arch: &Option<Architecture>) -> Option<Dcc> {
        self.dcc(&CommandLineTool::DCC64, arch)
    }

    pub fn dccarm64ec(&self, arch: &Option<Architecture>) -> Option<Dcc> {
        self.dcc(&CommandLineTool::DCCARM64EC, arch)
    }

    pub fn brcc32(&self, arch: &Option<Architecture>) -> Option<Brcc> {
        CommandLineTool::BRCC32
            .which(self.product_info(), arch)
            .map(|path| Brcc::new(path))
    }

    pub fn environment_variables(&self, arch: &Architecture) -> envz::Result<Environment> {
        Environment::create(
            &self.product_info().reg_key()?,
            format!("Environment Variables{}", arch.reg_name_suffix()),
            false,
        )
    }

    pub const LIBRARY_PATH: &StringEntry = &StringEntry {
        name: "Search Path",
        is_expand: false,
    };
    pub const BROWSING_PATH: &StringEntry = &StringEntry {
        name: "Browsing Path",
        is_expand: false,
    };

    pub fn library(&self, platform: &Platform) -> envz::Result<Node> {
        Node::create(
            &self.product_info().reg_key()?,
            format!("Library\\{platform}"),
        )
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

    pub fn count(&self) -> usize {
        self.items.len()
    }
}

pub fn find() -> Result<Installations> {
    let mut installs = Installations::new();
    #[cfg(windows)]
    {
        for path in [
            r"Software\Borland\BDS",
            r"Software\CodeGear\BDS",
            r"Software\Embarcadero\BDS",
        ] {
            let Ok(bds) = CURRENT_USER.open(path) else {
                continue;
            };
            for version in bds.keys()? {
                installs.push(Installation::new(PathBuf::from(path), version)?);
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
