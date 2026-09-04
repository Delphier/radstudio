use std::{
    ffi::{OsStr, OsString},
    fs::File,
    path::Path,
};

use tempfile::{NamedTempFile, TempPath};
use xmltree::{Element, EmitterConfig, XMLNode};

use super::consts::*;
use crate::Platform;

type TempPaths = Vec<TempPath>;
type PatchItems = Vec<PatchItem>;

struct PatchItem {
    name: &'static str,
    value: XMLNode,
    is_global: bool,
    is_cleanup: bool,
}

pub fn patch_project_file(
    platform: &Option<Platform>,
    options: &super::Options,
) -> crate::Result<Option<TempPaths>> {
    let ext = options.file.extension().unwrap_or_default();
    if ext.eq_ignore_ascii_case(EXT_GROUPPROJ) {
        patch_groupproj(platform, options, ext)
    } else if ext.eq_ignore_ascii_case(EXT_DPROJ) || ext.eq_ignore_ascii_case(EXT_CBPROJ) {
        patch_proj(platform, options, ext)
    } else {
        Ok(None)
    }
}

fn patch_proj(
    platform: &Option<Platform>,
    options: &super::Options,
    ext: impl AsRef<OsStr>,
) -> crate::Result<Option<TempPaths>> {
    let mut patches = PatchItems::new();

    if let Some(arch) = &options.preferred_tool_architecture {
        patches.push(PatchItem {
            name: DCC_USE_MSBUILD_EXTERNALLY,
            value: XMLNode::Text("true".to_string()),
            is_global: true,
            is_cleanup: true,
        });

        patches.push(PatchItem {
            name: DCC_PREFERRED_TOOL_ARCHITECTURE,
            value: XMLNode::Text(arch.to_string()),
            is_global: true,
            is_cleanup: true,
        });
    }

    let version_info = options.version_info.to_string();
    if !version_info.is_empty() {
        patches.push(PatchItem {
            name: VERINFO_INCLUDE_VERINFO,
            value: XMLNode::Text("true".to_string()),
            is_global: true,
            is_cleanup: true,
        });

        patches.push(PatchItem {
            name: VERINFO_KEYS,
            value: XMLNode::Text(version_info),
            is_global: true,
            is_cleanup: true,
        });
    }

    if patches.is_empty() && options.config.is_none() && platform.is_none() {
        return Ok(None);
    };

    let mut root = Element::parse(File::open(&options.file)?)?;

    let mut pg_first = match root.take_child(PROPERTY_GROUP) {
        Some(e) if e.attributes.is_empty() => e,
        _ => return Ok(None),
    };

    let pg_base_index = match root.children.iter().position(|node| matches!(node, XMLNode::Element(e) if e.matches(PROPERTY_GROUP) && e.attributes.get(CONDITION).map(String::as_str) == Some("'$(Base)'!=''"))){
        Some(i) => i,
        None => return Ok(None)
    };
    let mut pg_base = match root.children.remove(pg_base_index) {
        XMLNode::Element(e) => e,
        _ => return Ok(None),
    };

    let config = match resolve_prop_value(CONFIG, options.config.clone(), &pg_first, &mut patches) {
        Some(s) => s,
        None => return Ok(None),
    };

    let platform = match resolve_prop_value(
        PLATFORM,
        platform.as_ref().map(|p| p.to_string()),
        &pg_first,
        &mut patches,
    ) {
        Some(s) => s,
        None => return Ok(None),
    };

    let pg_last = find_property_group(&root, &config, &platform);
    let pre_build_user = pg_last
        .and_then(|e| get_prop_value(PRE_BUILD_EVENT, e))
        .unwrap_or_default();
    let post_build_user = pg_last
        .and_then(|e| get_prop_value(POST_BUILD_EVENT, e))
        .unwrap_or_default();
    let project_name = options.file.file_stem().unwrap_or_default().display();

    patches.push(PatchItem {
        name: PRE_BUILD_EVENT,
        value: XMLNode::CData(indoc::formatdoc! {r#"
            (IF EXIST "$(BRCC_OutputDir)$(MSBuildProjectName).res" COPY "$(BRCC_OutputDir)$(MSBuildProjectName).res" "$(BRCC_OutputDir){project_name}.res" /Y)
            (IF EXIST "$(MSBuildProjectName)Resource.rc" COPY "$(MSBuildProjectName)Resource.rc" "{project_name}Resource.rc" /Y)
            (IF EXIST "$(MSBuildProjectName).dres" COPY "$(MSBuildProjectName).dres" "{project_name}.dres" /Y)
            {pre_build_user}
            $({PRE_BUILD_EVENT})"#
        }),
        is_global: false,
        is_cleanup: true,
    });

    patches.push(PatchItem {
        name: POST_BUILD_EVENT,
        value: XMLNode::CData(indoc::formatdoc! {r#"
            {post_build_user}
            $({POST_BUILD_EVENT})
            DEL "$(BRCC_OutputDir)$(MSBuildProjectName).res"
            DEL "$(MSBuildProjectName)Resource.rc"
            DEL "$(MSBuildProjectName).dres"
            DEL "$(OutputDir)$(MSBuildProjectName).cmds" "#
        }),
        is_global: false,
        is_cleanup: true,
    });

    for item in patches {
        let target = match item.is_global {
            true => &mut pg_first,
            false => &mut pg_base,
        };
        remove_elements(target, &[item.name]);
        add_element(target, item.name, item.value);
        if item.is_cleanup {
            remove_elements(&mut root, &[item.name]);
        }
    }

    let output = create_temp_project_file(&options.file, ext)?;
    root.children
        .insert(pg_base_index, XMLNode::Element(pg_base));
    root.children.insert(0, XMLNode::Element(pg_first));
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
    _platform: &Option<Platform>,
    _options: &super::Options,
    _ext: impl AsRef<OsStr>,
) -> crate::Result<Option<TempPaths>> {
    // TODO
    Ok(None)
}

fn get_prop_value(name: &'static str, parent: &Element) -> Option<String> {
    parent
        .get_child(name)
        .and_then(|e| e.get_text())
        .map(|s| s.to_string())
}

fn resolve_prop_value(
    name: &'static str,
    value: Option<String>,
    parent: &Element,
    patches: &mut PatchItems,
) -> Option<String> {
    match value {
        Some(s) => {
            patches.push(PatchItem {
                name,
                value: XMLNode::Text(s.clone()),
                is_global: true,
                is_cleanup: false,
            });
            Some(s)
        }
        None => get_prop_value(name, parent),
    }
}

fn find_property_group<'a>(
    parent: &'a Element,
    config: &str,
    platform: &str,
) -> Option<&'a Element> {
    parent.children.iter().find_map(|node| match node {
        XMLNode::Element(e)
            if e.matches(PROPERTY_GROUP)
                && e.attributes.get(CONDITION)
                    == Some(&format!(
                        "'$(Config)'=='{config}' And '$(Platform)'=='{platform}'"
                    )) =>
        {
            Some(e)
        }
        _ => None,
    })
}

fn add_element(parent: &mut Element, name: &str, xmlnode: XMLNode) {
    let mut element = Element::new(name);
    element.children.push(xmlnode);
    parent.children.push(XMLNode::Element(element));
}

fn remove_elements(parent: &mut Element, names: &[&str]) {
    parent.children.retain_mut(|node| {
        let XMLNode::Element(e) = node else {
            return true;
        };
        if names.contains(&e.name.as_str()) {
            return false;
        };
        remove_elements(e, names);
        true
    });
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
