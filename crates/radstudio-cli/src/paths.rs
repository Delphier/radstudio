use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};
use radstudio::{Installation, Platforms};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, clap::Subcommand)]
pub(crate) enum PathsCmd {
    /// Append a item to the end of paths (skipped if already present)
    #[command(aliases = ["push", "append"])]
    Add { item: OsString },
    /// Prepend a item to the front of paths
    Insert { item: OsString },
    /// Remove item from paths
    #[command(aliases = ["rm", "delete", "del"])]
    Remove { item: OsString },
}

impl PathsCmd {
    pub fn execute(
        title: impl AsRef<str>,
        entry: &envz::registry::StringEntry,
        cmd: &Option<Self>,
        installation: &Installation,
        platforms: Platforms,
    ) -> envz::Result<()> {
        for platform in platforms {
            let node = installation.library(&platform)?;
            match cmd {
                Some(Self::Add { item }) => node.paths_push(entry, item)?,
                Some(Self::Insert { item }) => node.paths_insert(entry, item)?,
                Some(Self::Remove { item }) => node.paths_remove(entry, item)?,
                None => print(
                    format!("{platform} {}", title.as_ref()),
                    &node.get_paths(entry)?,
                ),
            }
        }
        Ok(())
    }
}

pub fn print(title: impl AsRef<str>, paths: &Vec<PathBuf>) {
    println!("{}", title.as_ref());
    let mut table = Table::new();
    table
        .load_style(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["ID", "Item"]);
    for (i, path) in paths.iter().enumerate() {
        table.add_row(vec![(i + 1).to_string(), path.display().to_string()]);
    }
    println!("{table}")
}
