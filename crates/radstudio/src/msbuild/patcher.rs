use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    fs::File,
    path::Path,
};

use tempfile::{NamedTempFile, TempPath};
use xmltree::{Element, EmitterConfig, XMLNode};

use super::consts::*;
use crate::Platform;

pub struct FileInfo<'f, 'd> {
    original: &'f Path,
    base_dir: Option<&'d Path>,
}

impl<'f, 'd> FileInfo<'f, 'd> {
    pub fn new(original: &'f impl AsRef<Path>, base_dir: Option<&'d Path>) -> Self {
        Self {
            original: original.as_ref(),
            base_dir,
        }
    }

    fn path(&self) -> Cow<'_, Path> {
        match self.base_dir {
            Some(base) if self.original.is_relative() => Cow::Owned(base.join(self.original)),
            _ => Cow::Borrowed(self.original),
        }
    }

    fn parent(&self) -> Cow<'_, Path> {
        match self.base_dir {
            Some(base) if self.original.is_relative() => match self.original.parent() {
                Some(p) => Cow::Owned(base.join(p)),
                None => Cow::Borrowed(base),
            },
            _ => Cow::Borrowed(self.original.parent().unwrap_or_else(|| Path::new(""))),
        }
    }

    fn project_name(&self) -> &OsStr {
        self.original.file_stem().unwrap_or_default()
    }

    fn extension(&self) -> &OsStr {
        self.original.extension().unwrap_or_default()
    }

    fn create_temp_project_file(&self, ext: impl AsRef<OsStr>) -> std::io::Result<NamedTempFile> {
        let mut prefix = self.project_name().to_owned();
        prefix.push(".");

        let mut suffix = OsString::from(".");
        suffix.push(ext);

        tempfile::Builder::new()
            .prefix(&prefix)
            .suffix(&suffix)
            .tempfile_in(self.parent())
    }
}

type TempPaths = Vec<TempPath>;
type PatchItems = Vec<PatchItem>;

struct PatchItem {
    name: &'static str,
    value: XMLNode,
    is_global: bool,
    is_cleanup: bool,
}

pub fn patch_project_file(
    file: &FileInfo,
    platform: &Option<Platform>,
    options: &super::Options,
) -> crate::Result<Option<TempPaths>> {
    let ext = file.extension();
    if ext.eq_ignore_ascii_case(EXT_GROUPPROJ) {
        patch_groupproj(file, platform, options, ext)
    } else if ext.eq_ignore_ascii_case(EXT_DPROJ) || ext.eq_ignore_ascii_case(EXT_CBPROJ) {
        patch_proj(file, platform, options, ext)
    } else {
        Ok(None)
    }
}

fn patch_groupproj(
    file: &FileInfo,
    platform: &Option<Platform>,
    options: &super::Options,
    ext: impl AsRef<OsStr>,
) -> crate::Result<Option<TempPaths>> {
    let mut root = Element::parse(File::open(file.path())?)?;
    let Some(ig) = root.get_mut_child(ITEM_GROUP) else {
        return Ok(None);
    };

    let base_dir = &file.parent();
    let mut result = TempPaths::new();
    let mut projects = Vec::new();

    for node in ig.children.iter_mut() {
        if let XMLNode::Element(e) = node
            && e.matches(PROJECTS)
            && let Some(original) = e.attributes.get(INCLUDE)
        {
            let subfile = FileInfo::new(original, Some(base_dir));
            if let Some(temps) = patch_project_file(&subfile, platform, options)? {
                let project = temps[0].display().to_string();
                e.attributes.insert(INCLUDE.to_string(), project.clone());
                projects.push(project);
                result.extend(temps);
            } else {
                projects.push(original.to_string());
            }
        }
    }

    if result.is_empty() {
        return Ok(None);
    }

    remove_elements(&mut root, &[TARGET]);
    let mut targets = Vec::new();
    for project in projects {
        let name = Path::new(&project)
            .file_stem()
            .unwrap_or_default()
            .display()
            .to_string()
            .replace(".", "_");

        let mut msbuild = Element::new(MSBUILD);
        msbuild.attributes.insert(PROJECTS.to_string(), project);

        let mut target = Element::new(TARGET);
        target.attributes.insert(NAME.to_string(), name.clone());
        target.children.push(XMLNode::Element(msbuild));

        root.children.push(XMLNode::Element(target));
        targets.push(name);
    }
    let mut call_target = Element::new(CALL_TARGET);
    call_target
        .attributes
        .insert(TARGETS.to_string(), targets.join(";"));
    let mut target = Element::new(TARGET);
    target
        .attributes
        .insert(NAME.to_string(), BUILD.to_string());
    target.children.push(XMLNode::Element(call_target));
    root.children.push(XMLNode::Element(target));

    let output = file.create_temp_project_file(ext)?;
    xml_write_file(&root, &output)?;

    let output = output.into_temp_path();
    let local = output.with_added_extension("local");
    result.insert(0, TempPath::try_from_path(local)?);
    result.insert(0, output);
    Ok(Some(result))
}

fn patch_proj(
    file: &FileInfo,
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

    let mut root = Element::parse(File::open(file.path())?)?;

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
    let project_name = file.project_name().display();

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

    root.children
        .insert(pg_base_index, XMLNode::Element(pg_base));
    root.children.insert(0, XMLNode::Element(pg_first));
    let output = file.create_temp_project_file(ext)?;
    xml_write_file(&root, &output)?;

    let local = output.path().with_added_extension("local");
    Ok(Some(vec![
        output.into_temp_path(),
        TempPath::try_from_path(local)?,
    ]))
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

fn xml_write_file(element: &Element, file: impl std::io::Write) -> Result<(), xmltree::Error> {
    element.write_with_config(
        file,
        EmitterConfig::new()
            .write_document_declaration(false)
            .perform_indent(true)
            .indent_string("    "),
    )
}
