use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};
use envz::registry::{Node, StringEntry};
use radstudio::{Installation, Platform, Platforms};
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
        cmd: &Option<Self>,
        cpp: bool,
        installation: &Installation,
        platforms: Platforms,
        entry: &envz::registry::StringEntry,
        title: impl AsRef<str>,
    ) -> envz::Result<()> {
        for platform in &platforms {
            let node = &if cpp {
                installation.cpp_paths(platform)?
            } else {
                installation.library(platform)?
            };
            let title = format!("{platform} {}", title.as_ref());
            if cpp
                && (platform == &Platform::Win32)
                && installation.product_info().supports_clang32()
            {
                let entry = &StringEntry::new(format!("{}_Clang32", entry.name), false);
                Self::exec(cmd, node, entry, &format!("{title} (Modern C++ Compiler)"))?;
            }
            Self::exec(cmd, node, entry, &title)?;
        }
        Ok(())
    }

    fn exec(cmd: &Option<Self>, node: &Node, entry: &StringEntry, title: &str) -> envz::Result<()> {
        match cmd {
            Some(Self::Add { item }) => node.paths_push(entry, item)?,
            Some(Self::Insert { item }) => node.paths_insert(entry, item)?,
            Some(Self::Remove { item }) => node.paths_remove(entry, item)?,
            None => print(title, &node.get_paths(entry)?),
        }
        Ok(())
    }
}

pub fn print(title: impl AsRef<str>, paths: &[PathBuf]) {
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
