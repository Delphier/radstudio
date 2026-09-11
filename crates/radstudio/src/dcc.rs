use std::{path::PathBuf, sync::OnceLock};

pub struct Arg {
    pub ident: &'static str,
    pub name: &'static str,
    pub msbuild: &'static str,
    pub value: String,
}

impl Arg {
    pub fn is_output_dir(&self) -> bool {
        self.ident.ends_with("output_dir")
    }
}

#[derive(Debug, radstudio_macros::DccOptions, clap::Args)]
pub struct Options {
    /// Specify the filename for compiling
    pub file: PathBuf,

    /// Do not load default dcc*.cfg file
    #[dcc(name = "--no-config")]
    #[arg(long)]
    no_config: bool,

    /// Define conditionals
    #[dcc(name = "-D", msbuild = "DCC_Define")]
    #[arg(short, long)]
    define: Vec<String>,

    /// Unit search directories
    //  MSBuild: DCC_UnitSearchPath = All search dirs
    #[dcc(name = "-U")]
    #[arg(long)]
    unit_search_dirs: Option<String>,

    /// Resource search directories
    #[dcc(name = "-R")]
    #[arg(long)]
    resource_search_dirs: Option<String>,

    /// Include search directories
    #[dcc(name = "-I")]
    #[arg(long)]
    include_search_dirs: Option<String>,

    /// Build all units
    #[dcc(name = "-B")]
    #[arg(short, long)]
    build: bool,

    /// Quiet compile
    #[dcc(name = "-Q")]
    #[arg(short, long)]
    quiet: bool,

    /// EXE/DLL output directory
    #[dcc(name = "-E", msbuild = "DCC_ExeOutput")]
    #[arg(long)]
    output_dir: Option<String>,

    /// Unit .dcu output directory
    #[dcc(name = "-NU", msbuild = "DCC_DcuOutput")]
    #[arg(long)]
    unit_output_dir: Option<String>,

    /// Package .bpl output directory
    #[dcc(name = "-LE", msbuild = "DCC_BplOutput")]
    #[arg(long)]
    package_bpl_output_dir: Option<String>,

    /// Package .dcp output directory
    #[dcc(name = "-LN", msbuild = "DCC_DcpOutput")]
    #[arg(long)]
    package_dcp_output_dir: Option<String>,

    /// Generate all C++Builder files
    //  MSBuild: <DCC_CBuilderOutput>All</DCC_CBuilderOutput>
    #[dcc(name = "-JL")]
    #[arg(long)]
    pub cpp: bool,

    /// Generate COFF-format C++ files
    #[dcc(name = "-jf:coffi")]
    #[arg(long)]
    cpp_win64x: bool,

    /// C++ .bpi output directory
    #[dcc(name = "-NB", msbuild = "DCC_BpiOutput")]
    #[arg(long)]
    cpp_bpi_output_dir: Option<String>,

    /// C++ .hpp output directory
    #[dcc(name = "-NH", msbuild = "DCC_HppOutput")]
    #[arg(long)]
    cpp_hpp_output_dir: Option<String>,

    /// C++ .obj/.lib output directory
    #[dcc(name = "-NO", msbuild = "DCC_ObjOutput")]
    #[arg(long)]
    cpp_obj_output_dir: Option<String>,

    /// Additional options to pass to the compiler
    //  MSBuild: DCC_AdditionalSwitches
    #[arg(last = true)]
    pub raw: Vec<String>,
}

impl Options {
    pub fn search_dirs(&self) -> Option<String> {
        let dirs: Vec<_> = [
            &self.unit_search_dirs,
            &self.resource_search_dirs,
            &self.include_search_dirs,
        ]
        .into_iter()
        .filter_map(|o| o.as_deref())
        .collect();

        if dirs.is_empty() {
            None
        } else {
            Some(dirs.join(";"))
        }
    }
}

pub struct Dcc {
    path: PathBuf,
    supports_command_line_compilation: OnceLock<bool>,
}

impl Dcc {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            supports_command_line_compilation: OnceLock::new(),
        }
    }

    pub fn supports_command_line_compilation(&self) -> bool {
        *self.supports_command_line_compilation.get_or_init(|| {
            const UNSUPPORTED_MSG: &str =
                "This version of the product does not support command line compiling.";
            std::process::Command::new(&self.path)
                .arg("--version")
                .output()
                .map(|o| !String::from_utf8_lossy(&o.stdout).starts_with(UNSUPPORTED_MSG))
                .unwrap_or_default()
        })
    }

    pub fn execute(&self, options: &Options) -> std::io::Result<std::process::ExitStatus> {
        let defaults = [
            "-AGenerics.Collections=System.Generics.Collections;Generics.Defaults=System.Generics.Defaults;WinTypes=Windows;WinProcs=Windows;DbiTypes=BDE;DbiProcs=BDE;DbiErrs=BDE",
            "-NSWinapi;System.Win;Data.Win;Datasnap.Win;Web.Win;Soap.Win;Xml.Win;Bde;System;Xml;Data;Datasnap;Web;Soap;Vcl;Vcl.Imaging;Vcl.Touch;Vcl.Samples;Vcl.Shell;IBX;VclTee",
        ];
        let mut cmd = std::process::Command::new(&self.path);
        cmd.arg(&options.file).args(defaults);

        for arg in options.data() {
            if !arg.name.is_empty() {
                if arg.value.is_empty() {
                    cmd.arg(&arg.name);
                } else {
                    cmd.arg(format!("{}\"{}\"", arg.name, arg.value));
                }
            }
            if arg.is_output_dir() {
                std::fs::create_dir_all(arg.value)?;
            }
        }

        cmd.args(&options.raw).status()
    }
}
