use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};
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

pub fn print(title: String, paths: &Vec<PathBuf>) {
    println!("{title}");
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
