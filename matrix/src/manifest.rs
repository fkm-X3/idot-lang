use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub license: Option<String>,
    pub dependencies: HashMap<String, DepEntry>,
}

#[derive(Debug, Clone)]
pub struct DepEntry {
    pub git: Option<String>,
    pub tag: Option<String>,
    pub branch: Option<String>,
}

impl Manifest {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let mut name = String::new();
        let mut version = String::new();
        let mut license = None;
        let mut dependencies = HashMap::new();
        let mut in_deps = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line == "[project]" {
                in_deps = false;
                continue;
            }
            if line == "[dependencies]" {
                in_deps = true;
                continue;
            }

            if in_deps {
                if let Some((dep_name, rest)) = line.split_once('=') {
                    let dep_name = dep_name.trim().to_string();
                    let rest = rest.trim().trim_start_matches('{').trim_end_matches('}');
                    let mut git = None;
                    let mut tag = None;
                    let mut branch = None;
                    for part in rest.split(',') {
                        let part = part.trim();
                        if let Some((k, v)) = part.split_once('=') {
                            let k = k.trim();
                            let v = v.trim().trim_matches('"');
                            match k {
                                "git" => git = Some(v.to_string()),
                                "tag" => tag = Some(v.to_string()),
                                "branch" => branch = Some(v.to_string()),
                                _ => {}
                            }
                        }
                    }
                    dependencies.insert(dep_name, DepEntry { git, tag, branch });
                }
            } else if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim().trim_matches('"');
                match k {
                    "name" => name = v.to_string(),
                    "version" => version = v.to_string(),
                    "license" => license = Some(v.to_string()),
                    _ => {}
                }
            }
        }

        if name.is_empty() {
            return Err("Missing 'name' field in [project]".to_string());
        }
        if version.is_empty() {
            return Err("Missing 'version' field in [project]".to_string());
        }

        Ok(Manifest { name, version, license, dependencies })
    }

    pub fn template(name: &str) -> String {
        format!(
            r#"[project]
name = "{}"
version = "0.1.0"
license = "MIT"

[dependencies]
"#,
            name
        )
    }
}
