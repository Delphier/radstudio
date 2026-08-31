use std::{os::windows::process::CommandExt, path::PathBuf};

#[derive(Debug, clap::Args, serde::Serialize)]
pub struct Options {
    /// Specify the filename for compiling
    #[serde(skip)]
    file: PathBuf,

    /// Do not load default dcc*.cfg file
    #[serde(rename = "--no-config")]
    #[arg(long)]
    no_config: bool,

    /// Define conditionals
    #[serde(rename = "-D")]
    #[arg(short, long)]
    define: Vec<String>,

    /// Unit search directories
    #[serde(rename = "-U")]
    #[arg(long)]
    unit_search_dirs: Option<String>,

    /// Resource search directories
    #[serde(rename = "-R")]
    #[arg(long)]
    resource_search_dirs: Option<String>,

    /// Include search directories
    #[serde(rename = "-I")]
    #[arg(long)]
    include_search_dirs: Option<String>,

    /// Build all units
    #[serde(rename = "-B")]
    #[arg(short, long)]
    build: bool,

    /// Quiet compile
    #[serde(rename = "-Q")]
    #[arg(short, long)]
    quiet: bool,

    /// EXE/DLL output directory
    #[serde(rename = "-E")]
    #[arg(long)]
    output_dir: Option<String>,

    /// Unit .dcu output directory
    #[serde(rename = "-NU")]
    #[arg(long)]
    unit_output_dir: Option<String>,

    /// Package .bpl output directory
    #[serde(rename = "-LE")]
    #[arg(long)]
    package_bpl_output_dir: Option<String>,

    /// Package .dcp output directory
    #[serde(rename = "-LN")]
    #[arg(long)]
    package_dcp_output_dir: Option<String>,

    /// Generate all C++Builder files
    #[serde(rename = "-JL")]
    #[arg(long)]
    cpp: bool,

    /// Generate COFF-format C++ files
    #[serde(rename = "-jf:coffi")]
    #[arg(long)]
    cpp_win64x: bool,

    /// C++ .bpi output directory
    #[serde(rename = "-NB")]
    #[arg(long)]
    cpp_bpi_output_dir: Option<String>,

    /// C++ .hpp output directory
    #[serde(rename = "-NH")]
    #[arg(long)]
    cpp_hpp_output_dir: Option<String>,

    /// C++ .obj/.lib output directory
    #[serde(rename = "-NO")]
    #[arg(long)]
    cpp_obj_output_dir: Option<String>,

    /// Additional options to pass to the compiler
    #[serde(skip)]
    #[arg(long = "options")]
    raw: Option<String>,
}

fn format_option(name: impl AsRef<str>, value: impl AsRef<str>) -> String {
    format!(r#"{}"{}""#, name.as_ref(), value.as_ref())
}
impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut options = vec![
            "-AGenerics.Collections=System.Generics.Collections;Generics.Defaults=System.Generics.Defaults;WinTypes=Windows;WinProcs=Windows;DbiTypes=BDE;DbiProcs=BDE;DbiErrs=BDE".to_string(),
            "-NSWinapi;System.Win;Data.Win;Datasnap.Win;Web.Win;Soap.Win;Xml.Win;Bde;System;Xml;Data;Datasnap;Web;Soap;Vcl;Vcl.Imaging;Vcl.Touch;Vcl.Samples;Vcl.Shell;IBX;VclTee".to_string(),
        ];
        if let Ok(serde_json::Value::Object(obj)) = serde_json::to_value(self) {
            for (k, v) in obj {
                match v {
                    serde_json::Value::Bool(true) => options.push(k),
                    serde_json::Value::String(s) => options.push(format_option(k, s)),
                    serde_json::Value::Array(a) => {
                        for i in a {
                            if let serde_json::Value::String(s) = i {
                                options.push(format_option(&k, s))
                            }
                        }
                    }
                    _ => {}
                };
            }
        }
        if let Some(s) = &self.raw {
            options.push(s.clone());
        }
        f.write_str(&options.join(" "))
    }
}

pub struct Dcc {
    path: PathBuf,
}

impl Dcc {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn execute(&self, options: &Options) -> std::io::Result<std::process::ExitStatus> {
        std::process::Command::new(&self.path)
            .arg(&options.file)
            .raw_arg(&options.to_string())
            .status()
    }
}
