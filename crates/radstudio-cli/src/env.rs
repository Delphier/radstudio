use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};
use radstudio::{Architectures, Installation};

#[derive(Debug, clap::Subcommand)]
pub(crate) enum EnvCmd {
    /// Set IDE environment variable
    #[command(alias = "add")]
    Set {
        /// Specify environment variable name
        name: String,
        /// Specify environment variable value
        value: String,
    },
    /// Remove IDE environment variable
    #[command(aliases = ["rm", "delete", "del"])]
    Remove {
        /// Specify environment variable name
        name: String,
    },
}

impl EnvCmd {
    pub fn execute(
        cmd: &Option<Self>,
        installation: &Installation,
        archs: Architectures,
    ) -> envz::Result<()> {
        for arch in archs {
            let env = installation.environment_variables(&arch)?;
            match cmd {
                Some(Self::Set { name, value }) => env.set(name, value)?,
                Some(Self::Remove { name }) => env.remove(name)?,
                None => {
                    println!("{}", arch.ide_name());
                    let mut table = Table::new();
                    table
                        .load_style(UTF8_FULL_CONDENSED)
                        .set_content_arrangement(ContentArrangement::Dynamic)
                        .set_header(vec!["Name", "Value"]);
                    for (name, value) in env.values()? {
                        table.add_row(vec![name, String::try_from(value)?]);
                    }
                    println!("{table}");
                }
            }
        }
        Ok(())
    }
}
