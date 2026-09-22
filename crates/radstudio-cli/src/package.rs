use crate::App;
use anyhow::Context;
use radstudio::package::PackageInfo;
use std::path::PathBuf;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum PackageCmd {
    /// Compile a design-time package and optionally install it
    Compile {
        /// Specify weather to install the package into the IDE
        #[arg(long)]
        install: bool,
        #[command(flatten)]
        options: radstudio::dcc::Options,
    },

    /// Register a design-time package
    Register {
        /// Specify the full path of the bpl file to register
        bpl_path: PathBuf,
        /// Specify the description that appears in the registry
        description: String,
    },

    /// Unregister a design-time package
    Unregister {
        /// Specify the full path of the bpl file to unregister
        bpl_path: PathBuf,
    },
}

impl PackageCmd {
    pub fn execute(&self, app: &App) -> anyhow::Result<()> {
        let installation = app.installation();
        let arch = &match &app.global.platform {
            Some(p) if app.global.architecture.is_none() => installation
                .product_info()
                .ide_architectures()
                .into_iter()
                .find(|a| &a.platform() == p)
                .context(format!(
                    "the current IDE does not support {p} design-time packages"
                ))?,
            _ => app.ide_architecture()?,
        };

        match self {
            Self::Compile { install, options } => {
                app.dcc_execute(&arch.platform().command_line_tool(), options)?;
                if *install
                    && let Some(bpl_output_dir) = &options.package_bpl_output_dir
                    && let Some(file_name) = options.file.file_name()
                {
                    let bpl_output_dir = std::env::current_dir()?.join(bpl_output_dir);
                    let bpl_path = bpl_output_dir.join(file_name).with_extension("bpl");
                    let description = PackageInfo::from_file(&options.file)?.description;
                    installation.register_package(arch, bpl_path, description)?;
                }
            }
            Self::Register {
                bpl_path,
                description,
            } => installation.register_package(arch, bpl_path, description)?,
            Self::Unregister { bpl_path } => installation.unregister_pacakge(arch, bpl_path)?,
        }

        Ok(())
    }
}
