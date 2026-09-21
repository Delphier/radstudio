use crate::{INSTALLATIONS, LATEST_INSTALLATION};
use comfy_table::{ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};
use heck::ToTitleCase;
use radstudio::{Installation, ProductInfo};

pub fn print(installation: Option<&Installation>, json: bool) -> anyhow::Result<()> {
    match installation {
        Some(i) => print_installation(i, None, json)?,
        None => {
            if INSTALLATIONS.len() == 1 {
                print_installation(&LATEST_INSTALLATION, None, json)?;
            } else {
                if json {
                    let data = INSTALLATIONS
                        .iter()
                        .map(|i| i.product_info().data())
                        .collect::<Result<Vec<_>, _>>()?;
                    println!("{}", serde_json::to_string_pretty(&data)?);
                } else {
                    for (id, i) in INSTALLATIONS.iter().enumerate() {
                        print_installation(i, Some(id + 1), false)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn print_installation(
    installation: &Installation,
    id: Option<usize>,
    json: bool,
) -> anyhow::Result<()> {
    if json {
        let data = serde_json::to_string_pretty(&vec![installation.product_info().data()?])?;
        Ok(println!("{data}"))
    } else {
        id.inspect(|id| print!("{id}. "));
        println!("{}", installation.product_info().display_name());
        print_product_info(installation.product_info())
    }
}

fn print_product_info(pi: &ProductInfo) -> anyhow::Result<()> {
    let mut table = Table::new();
    table
        .load_style(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Property", "Value"]);

    for (name, value) in pi.data()? {
        table.add_row(vec![name.to_title_case(), value.to_string()]);
    }

    for arch in &pi.architectures() {
        let arch_name = format!("{arch}: ");
        table
            .add_row(vec![
                format!("{arch_name}Bin Dir"),
                pi.bin_dir(arch).display().to_string(),
            ])
            .add_row(vec![
                format!("{arch_name}rsvars.bat"),
                pi.rsvars_bat(arch).display().to_string(),
            ])
            .add_row(vec![
                format!("{arch_name}bds.exe"),
                pi.bds_exe(arch).display().to_string(),
            ])
            .add_row(vec![
                format!("{arch_name}Command-line Tools"),
                format!("{:?}", pi.command_line_tools(arch)),
            ]);
    }

    println!("{table}");
    Ok(())
}
