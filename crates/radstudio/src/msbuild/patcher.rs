use std::{
    ffi::{OsStr, OsString},
    fs::File,
    path::Path,
};

use tempfile::{NamedTempFile, TempPath};
use xmltree::{Element, EmitterConfig, XMLNode};

use super::consts::*;
use crate::{Architecture, Platform};

type TempPaths = Vec<TempPath>;
type PatchItems = Vec<PatchItem>;

struct PatchItem {
    name: &'static str,
    value: String,
    is_cleanup: bool,
}

pub fn patch_project_file(
    arch: &Option<Architecture>,
    platform: &Option<Platform>,
    options: &super::Options,
) -> crate::Result<Option<TempPaths>> {
    let mut patches = PatchItems::new();

    if let Some(config) = &options.config {
        patches.push(PatchItem {
            name: CONFIG,
            value: config.to_owned(),
            is_cleanup: false,
        });
    };

    if let Some(platform) = platform {
        patches.push(PatchItem {
            name: PLATFORM,
            value: platform.to_string(),
            is_cleanup: false,
        });
    }

    let version_info = options.version_info.to_string();
    if !version_info.is_empty() {
        patches.push(PatchItem {
            name: VERINFO_INCLUDE_VERINFO,
            value: "true".to_string(),
            is_cleanup: true,
        });

        patches.push(PatchItem {
            name: VERINFO_KEYS,
            value: version_info,
            is_cleanup: true,
        });
    }

    if let Some(arch) = arch {
        patches.push(PatchItem {
            name: DCC_USE_MSBUILD_EXTERNALLY,
            value: "true".to_string(),
            is_cleanup: true,
        });

        patches.push(PatchItem {
            name: DCC_PREFERRED_TOOL_ARCHITECTURE,
            value: arch.to_string(),
            is_cleanup: true,
        });
    }

    if patches.is_empty() {
        return Ok(None);
    };

    let ext = options.file.extension().unwrap_or_default();
    if ext.eq_ignore_ascii_case(EXT_GROUPPROJ) {
        patch_groupproj(&options.file, patches, ext)
    } else if ext.eq_ignore_ascii_case(EXT_DPROJ) || ext.eq_ignore_ascii_case(EXT_CBPROJ) {
        patch_dproj(&options.file, patches, ext)
    } else {
        Ok(None)
    }
}

fn patch_dproj(
    original: &Path,
    patches: PatchItems,
    ext: impl AsRef<OsStr>,
) -> crate::Result<Option<TempPaths>> {
    let mut root = Element::parse(File::open(original)?)?;
    let index = root.children.iter().position(|node| matches!(node, XMLNode::Element(child) if child.name.eq_ignore_ascii_case(PROPERTY_GROUP) && child.attributes.is_empty()));
    let mut pg = match index {
        Some(i) if let XMLNode::Element(pg) = root.children.remove(i) => pg,
        _ => Element::new(PROPERTY_GROUP),
    };

    for item in patches {
        if item.is_cleanup {
            remove_elements(&mut root, &[item.name]);
        }
        remove_elements(&mut pg, &[item.name]);
        add_element(&mut pg, item.name, item.value);
    }

    let project_name = original.file_stem().unwrap_or_default().display();
    let mut element = Element::new(PRE_BUILD_EVENT);
    element.children.push(XMLNode::CData(format!(
        r#"COPY "$(BRCC_OutputDir)$(MSBuildProjectName).res" "$(BRCC_OutputDir){project_name}.res" /Y
           DEL  "$(BRCC_OutputDir)$(MSBuildProjectName).res"
           COPY "$(MSBuildProjectName)Resource.rc" "{project_name}Resource.rc" /Y
           DEL  "$(MSBuildProjectName)Resource.rc"
           COPY "$(MSBuildProjectName).dres" "{project_name}.dres" /Y
           DEL  "$(MSBuildProjectName).dres"
           $({PRE_BUILD_EVENT})"#
    )));
    remove_elements(&mut pg, &[PRE_BUILD_EVENT]);
    pg.children.push(XMLNode::Element(element));

    //todo: remove OUTPUT_DIR\.cmds in POST_BUILD_EVENT

    let output = create_temp_project_file(original, ext)?;
    root.children.insert(0, XMLNode::Element(pg));
    root.write_with_config(
        &output,
        EmitterConfig::new()
            .perform_indent(true)
            .indent_string("    ")
            .write_document_declaration(false),
    )?;

    let local = output.path().with_added_extension("local");
    Ok(Some(vec![
        output.into_temp_path(),
        TempPath::try_from_path(local)?,
    ]))
}

fn patch_groupproj(
    _original: &Path,
    _patches: PatchItems,
    _ext: impl AsRef<OsStr>,
) -> crate::Result<Option<TempPaths>> {
    // todo
    Ok(None)
}

fn set_element_text(element: &mut Element, text: impl AsRef<str>) {
    element.children.clear();
    element
        .children
        .push(XMLNode::Text(text.as_ref().to_owned()));
}

fn add_element(parent: &mut Element, name: &str, text: impl AsRef<str>) {
    let mut element = Element::new(name);
    set_element_text(&mut element, text);
    parent.children.push(XMLNode::Element(element));
}

fn remove_elements(parent: &mut Element, names: &[&str]) {
    parent.children.retain(|node| !matches!(node, XMLNode::Element(child) if names.iter().any(|name| name.eq_ignore_ascii_case(&child.name))));
    for node in parent.children.iter_mut() {
        if let XMLNode::Element(child) = node {
            remove_elements(child, names);
        }
    }
}

pub(crate) fn create_temp_project_file(
    original: &Path,
    ext: impl AsRef<OsStr>,
) -> std::io::Result<NamedTempFile> {
    let mut prefix = original.file_stem().unwrap_or_default().to_owned();
    prefix.push(".");

    let mut suffix = OsString::from(".");
    suffix.push(ext);

    tempfile::Builder::new()
        .prefix(&prefix)
        .suffix(&suffix)
        .tempfile_in(original.parent().unwrap_or_else(|| Path::new(".")))
}
