use anyhow::{format_err, Context, Result};
use std::{collections::HashSet, io::Read};
use strum::IntoEnumIterator;

mod npm;

#[derive(Clone, Debug)]
pub struct JsExtension {
    name_: String,
    host_name_: String,
    root_url_: url::Url,
    package_url_template_: String,
    package_version_url_template_: String,
}

impl vouch_lib::extension::Extension for JsExtension {
    fn new() -> Self {
        Self {
            name_: "js".to_string(),
            host_name_: "npmjs.com".to_string(),
            root_url_: url::Url::parse("https://www.npmjs.com").unwrap(),
            package_url_template_: "https://www.npmjs.com/package/{{package_name}}/".to_string(),
            package_version_url_template_:
                "https://www.npmjs.com/package/{{package_name}}/v/{{package_version}}".to_string(),
        }
    }

    fn from_process(
        _process_path: &std::path::PathBuf,
        _extension_config_path: &std::path::PathBuf,
    ) -> Result<Self> {
        unimplemented!();
    }

    fn name(&self) -> String {
        self.name_.clone()
    }

    fn identify_local_dependancies(
        &self,
        working_directory: &std::path::PathBuf,
    ) -> Result<Vec<vouch_lib::extension::LocalDependancy>> {
        // Identify all dependancy definition files.
        let dependancy_files = match identify_dependancy_files(&working_directory) {
            Some(v) => v,
            None => return Ok(Vec::new()),
        };

        // Read all dependancies definitions files.
        let mut all_dependancies = HashSet::new();
        for dependancy_file in dependancy_files {
            // TODO: Handle all definition files.
            let dependancies: HashSet<vouch_lib::extension::LocalDependancy> =
                match dependancy_file.r#type {
                    DependancyFileType::Npm => npm::get_dependancies(&dependancy_file.path)?,
                };
            for dependancy in dependancies {
                all_dependancies.insert(dependancy);
            }
        }

        Ok(all_dependancies.into_iter().collect())
    }

    fn remote_package_metadata(
        &self,
        package_name: &str,
        package_version: &str,
        working_directory: &std::path::PathBuf,
    ) -> Result<vouch_lib::extension::RemotePackageMetadata> {
        let dependancy_files = identify_dependancy_files(&working_directory);
        let found_local_use = dependancy_files.is_some();

        // Query remote package registry for given package.
        let registry_package_url = get_package_url(&self, &package_name)?;
        let registry_package_version_url =
            get_package_version_url(&self, &package_name, &package_version)?;

        let registry_package_url = match &registry_package_url {
            Some(v) => v,
            None => {
                return Ok(vouch_lib::extension::RemotePackageMetadata {
                    found_local_use,
                    registry_host_name: Some(self.host_name_.clone()),
                    registry_package_url: registry_package_url.map(|x| x.to_string()),
                    registry_package_version_url: registry_package_version_url
                        .map(|x| x.to_string()),
                    source_code_url: None,
                    source_code_sha256: None,
                });
            }
        };

        let entry_json = get_registry_entry_json(&package_name)?;
        let source_code_url = get_source_code_url(&entry_json, &package_version)?;
        let source_code_sha256 = get_source_code_sha256(&entry_json, &package_version)?;

        Ok(vouch_lib::extension::RemotePackageMetadata {
            found_local_use,
            registry_host_name: Some(self.host_name_.clone()),
            registry_package_url: Some(registry_package_url.to_string()),
            registry_package_version_url: registry_package_version_url.map(|x| x.to_string()),
            source_code_url: Some(source_code_url.to_string()),
            source_code_sha256: Some(source_code_sha256),
        })
    }
}

fn get_package_url(extension: &JsExtension, package_name: &str) -> Result<Option<url::Url>> {
    // Example return value: https://www.npmjs.com/package/d3/
    let handlebars_registry = handlebars::Handlebars::new();
    let url = handlebars_registry.render_template(
        &extension.package_url_template_,
        &maplit::btreemap! {
            "package_name" => package_name,
        },
    )?;
    Ok(Some(url::Url::parse(url.as_str())?))
}

fn get_package_version_url(
    extension: &JsExtension,
    package_name: &str,
    package_version: &str,
) -> Result<Option<url::Url>> {
    // Example return value: https://www.npmjs.com/package/d3/v/6.5.0
    let handlebars_registry = handlebars::Handlebars::new();
    let url = handlebars_registry.render_template(
        &extension.package_version_url_template_,
        &maplit::btreemap! {
            "package_name" => package_name,
            "package_version" => package_version,
        },
    )?;
    Ok(Some(url::Url::parse(url.as_str())?))
}

fn get_registry_entry_json(package_name: &str) -> Result<serde_json::Value> {
    let handlebars_registry = handlebars::Handlebars::new();
    let json_url = handlebars_registry.render_template(
        "https://registry.npmjs.com/{{package_name}}",
        &maplit::btreemap! {"package_name" => package_name},
    )?;

    let mut result = reqwest::blocking::get(&json_url.to_string())?;
    let mut body = String::new();
    result.read_to_string(&mut body)?;

    Ok(serde_json::from_str(&body).context(format!("JSON was not well-formatted:\n{}", body))?)
}

fn get_source_code_url(
    registry_entry_json: &serde_json::Value,
    package_version: &str,
) -> Result<url::Url> {
    Ok(url::Url::parse(
        registry_entry_json["versions"][package_version]["dist"]["tarball"]
            .as_str()
            .ok_or(format_err!("Failed to parse package source code URL."))?,
    )?)
}

fn get_source_code_sha256(
    registry_entry_json: &serde_json::Value,
    package_version: &str,
) -> Result<String> {
    Ok(
        registry_entry_json["versions"][package_version]["dist"]["shasum"]
            .to_string()
            .replace("\"", ""),
    )
}

/// Package dependancy file types.
#[derive(Debug, Copy, Clone, strum_macros::EnumIter)]
enum DependancyFileType {
    Npm,
}

impl DependancyFileType {
    /// Return file name associated with dependancy type.
    pub fn file_name(&self) -> std::path::PathBuf {
        match self {
            Self::Npm => std::path::PathBuf::from("package.json"),
        }
    }
}

/// Package dependancy file type and file path.
#[derive(Debug, Clone)]
struct DependancyFile {
    r#type: DependancyFileType,
    path: std::path::PathBuf,
}

/// Returns a vector of identified package dependancy definition files.
///
/// Walks up the directory tree directory tree until the first positive result is found.
fn identify_dependancy_files(
    working_directory: &std::path::PathBuf,
) -> Option<Vec<DependancyFile>> {
    assert!(working_directory.is_absolute());
    let mut working_directory = working_directory.clone();

    loop {
        // If at least one target is found, assume package is present.
        let mut found_dependancy_file = false;

        let mut dependancy_files: Vec<DependancyFile> = Vec::new();
        for dependancy_file_type in DependancyFileType::iter() {
            let target_absolute_path = working_directory.join(dependancy_file_type.file_name());
            if target_absolute_path.is_file() {
                found_dependancy_file = true;
                dependancy_files.push(DependancyFile {
                    r#type: dependancy_file_type,
                    path: target_absolute_path,
                })
            }
        }
        if found_dependancy_file {
            return Some(dependancy_files);
        }

        // No need to move further up the directory tree after this loop.
        if working_directory == std::path::PathBuf::from("/") {
            break;
        }

        // Move further up the directory tree.
        working_directory.pop();
    }
    None
}
