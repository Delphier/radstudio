use crate::Platform;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};
use tempfile::NamedTempFile;

pub struct Bds {
    path: PathBuf,
}

impl Bds {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn execute(
        &self,
        platform: &Option<Platform>,
        options: &crate::msbuild::Options,
    ) -> crate::Result<ExitStatus> {
        let temps = crate::msbuild::patcher::patch_project_file(platform, options)?;
        let input: &Path = match &temps {
            Some(temps) => &temps[0],
            None => &options.file,
        };
        let output = NamedTempFile::new()?.into_temp_path();

        println!("Starting bds.exe and building...");
        let status = Command::new(&self.path)
            .arg(input)
            .arg("-b")
            .raw_arg(format!(r#"-o"{}""#, output.display()))
            .status()?;

        let mut indent = false;
        let reader = BufReader::new(File::open(output)?);
        for line in reader.lines() {
            let line = line?;
            if line.starts_with("  ") {
                indent = true;
                print!("{}", line.trim_start());
            } else {
                if indent {
                    println!("\n")
                };
                indent = false;
                println!("{}", line.trim_start_matches('\u{FEFF}'))
            }
        }

        Ok(status)
    }
}
