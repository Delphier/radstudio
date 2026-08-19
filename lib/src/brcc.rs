use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Options {
    /// Specify a resource script file (.rc) to be compiled
    pub input_rc: PathBuf,
    /// Specify the output resource .res file
    ///
    /// If omitted, creates the output .RES file with the same name as the input .rc file
    pub output_res: Option<PathBuf>,
}

pub struct Brcc {
    path: PathBuf,
}

impl Brcc {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn execute(&self, options: &Options) -> std::io::Result<std::process::ExitStatus> {
        let mut cmd = std::process::Command::new(&self.path);
        cmd.arg(&options.input_rc);
        if let Some(output) = &options.output_res {
            cmd.arg("-fo").arg(output);
        }
        cmd.status()
    }
}
