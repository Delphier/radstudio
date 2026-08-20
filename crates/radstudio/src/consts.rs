pub const SEATTLE: &'static str = "Seattle";
pub const BERLIN: &'static str = "Berlin";
pub const TOKYO: &'static str = "Tokyo";
pub const RIO: &'static str = "Rio";
pub const SYDNEY: &'static str = "Sydney";
pub const ALEXANDRIA: &'static str = "Alexandria";
pub const ATHENS: &'static str = "Athens";
pub const FLORENCE: &'static str = "Florence";

#[derive(Debug)]
pub(crate) struct VersionInfo {
    version: &'static str,
    pub product_version: &'static str,
    pub code_name: &'static str,
}

impl VersionInfo {
    pub fn new(version: &str) -> Option<&'static Self> {
        VERSIONS.iter().find(|x| x.version == version)
    }
}

// https://docwiki.embarcadero.com/Support/en/Supported_Versions
// https://docwiki.embarcadero.com/RADStudio/en/Compiler_Versions
// https://en.wikipedia.org/wiki/History_of_Delphi_(software)
pub(crate) static VERSIONS: &[VersionInfo] = &[
    VersionInfo {
        version: "2",
        product_version: "8",
        code_name: "",
    },
    VersionInfo {
        version: "3",
        product_version: "2005",
        code_name: "",
    },
    VersionInfo {
        version: "4",
        product_version: "2006",
        code_name: "",
    },
    VersionInfo {
        version: "5",
        product_version: "2007",
        code_name: "",
    },
    VersionInfo {
        version: "6",
        product_version: "2009",
        code_name: "",
    },
    VersionInfo {
        version: "7",
        product_version: "2010",
        code_name: "",
    },
    VersionInfo {
        version: "8.0",
        product_version: "XE",
        code_name: "",
    },
    VersionInfo {
        version: "9.0",
        product_version: "XE2",
        code_name: "",
    },
    VersionInfo {
        version: "10.0",
        product_version: "XE3",
        code_name: "",
    },
    VersionInfo {
        version: "11.0",
        product_version: "XE4",
        code_name: "",
    },
    VersionInfo {
        version: "12.0",
        product_version: "XE5",
        code_name: "",
    },
    VersionInfo {
        version: "14.0",
        product_version: "XE6",
        code_name: "",
    },
    VersionInfo {
        version: "15.0",
        product_version: "XE7",
        code_name: "",
    },
    VersionInfo {
        version: "16.0",
        product_version: "XE8",
        code_name: "",
    },
    VersionInfo {
        version: "17.0",
        product_version: "10",
        code_name: SEATTLE,
    },
    VersionInfo {
        version: "19.0",
        product_version: "10.1",
        code_name: BERLIN,
    },
    VersionInfo {
        version: "19.0",
        product_version: "10.2",
        code_name: TOKYO,
    },
    VersionInfo {
        version: "20.0",
        product_version: "10.3",
        code_name: RIO,
    },
    VersionInfo {
        version: "21.0",
        product_version: "10.4",
        code_name: SYDNEY,
    },
    VersionInfo {
        version: "22.0",
        product_version: "11",
        code_name: ALEXANDRIA,
    },
    VersionInfo {
        version: "23.0",
        product_version: "12",
        code_name: ATHENS,
    },
    VersionInfo {
        version: "37.0",
        product_version: "13",
        code_name: FLORENCE,
    },
];
