use std::{
    fs::File,
    path::{Path, PathBuf},
};

use tempfile::TempDir;
use uuid::Uuid;
use xmltree::{Element, XMLNode};

use crate::{
    Platform, dcc,
    msbuild::{add_text_element, consts::*, xml_write_file},
};

pub fn generator_project_file(
    file: &super::FileInfo,
    platform: &Option<Platform>,
    options: &dcc::Options,
) -> crate::Result<Option<(PathBuf, TempDir)>> {
    let ext = file.extension();
    let project_type = if ext.eq_ignore_ascii_case("dpk") {
        "Package"
    } else {
        return Ok(None);
    };

    let mut root = Element::new("Project");
    root.attributes.insert(
        "xmlns".to_string(),
        "http://schemas.microsoft.com/developer/msbuild/2003".to_string(),
    );

    let mut pg_first = Element::new(PROPERTY_GROUP);
    add_text_element(
        &mut pg_first,
        "ProjectGuid",
        format!("{{{:X}}}", Uuid::new_v4()),
    );
    add_text_element(&mut pg_first, "ProjectVersion", "99.9".to_string());
    add_text_element(
        &mut pg_first,
        "MainSource",
        file.path().display().to_string(),
    );
    add_text_element(&mut pg_first, CONFIG, "Release".to_string());
    if let Some(platform) = platform {
        add_text_element(&mut pg_first, PLATFORM, platform.to_string());
    };

    let mut pg_empty = Element::new(PROPERTY_GROUP);
    pg_empty.attributes.insert(
        CONDITION.to_string(),
        "'$(Config)'=='Base' or '$(Base)'!=''".to_string(),
    );

    let mut pg_base = Element::new(PROPERTY_GROUP);
    pg_base.attributes.insert(
        CONDITION.to_string(),
        PROPERTY_GROUP_BASE_CONDITION.to_string(),
    );

    let curdir = std::env::current_dir()?;

    for arg in options.data() {
        if !arg.msbuild.is_empty() {
            add_text_element(
                &mut pg_base,
                arg.msbuild,
                if arg.is_output_dir() {
                    absolute_dirs(&curdir, &arg.value)
                } else {
                    arg.value
                },
            );
        }
    }

    if let Some(search_dirs) = options.search_dirs() {
        add_text_element(
            &mut pg_base,
            "DCC_UnitSearchPath",
            absolute_dirs(&curdir, &search_dirs),
        );
    }

    if options.cpp {
        add_text_element(&mut pg_base, "DCC_CBuilderOutput", "All".to_string());
    };

    if !options.raw.is_empty() {
        add_text_element(
            &mut pg_base,
            "DCC_AdditionalSwitches",
            options.raw.join(" "),
        );
    }

    let mut pe = Element::new("ProjectExtensions");
    add_text_element(&mut pe, "Borland.ProjectType", project_type.to_string());

    root.children.push(XMLNode::Element(pg_first));
    root.children.push(XMLNode::Element(pg_empty));
    root.children.push(XMLNode::Element(pg_base));
    root.children.push(XMLNode::Element(pe));

    let tempdir = tempfile::tempdir()?;
    let output = tempdir
        .path()
        .join(file.project_name())
        .with_extension(EXT_DPROJ);
    xml_write_file(&root, File::create(&output)?)?;
    Ok(Some((output, tempdir)))
}

fn absolute_dirs(base_dir: impl AsRef<Path>, input: impl AsRef<str>) -> String {
    input
        .as_ref()
        .split(";")
        .map(|s| base_dir.as_ref().join(s).display().to_string())
        .collect::<Vec<_>>()
        .join(";")
}
