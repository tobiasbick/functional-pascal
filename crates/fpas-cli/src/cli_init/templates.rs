//! Canonical manifest and formatted FPAS source templates.

use std::path::PathBuf;

/// Root-relative template entry.
pub(super) struct TemplateFile {
    pub(super) relative_path: PathBuf,
    pub(super) content: String,
}

/// Template files paired with the root manifest selected for reporting.
pub(super) struct TemplatePlan {
    pub(super) manifest: PathBuf,
    pub(super) files: Vec<TemplateFile>,
}

/// Returns the runnable program scaffold.
pub(super) fn project(name: &str, identifier: &str) -> TemplatePlan {
    TemplatePlan {
        manifest: PathBuf::from(format!("{name}.fpasprj")),
        files: vec![
            file(".gitignore", gitignore()),
            file(
                &format!("{name}.fpasprj"),
                format!(
                    "[project]\nname = \"{name}\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n"
                ),
            ),
            file(
                "src/main.fpas",
                format!(
                    "program {identifier};\n\nuses Std.Console;\n\nbegin\n  WriteLn('Hello from {name}')\nend.\n"
                ),
            ),
        ],
    }
}

/// Returns the reusable library scaffold.
pub(super) fn library(name: &str, unit: &str) -> TemplatePlan {
    TemplatePlan {
        manifest: PathBuf::from(format!("{name}.fpasprj")),
        files: vec![
            file(".gitignore", gitignore()),
            file(
                &format!("{name}.fpasprj"),
                format!(
                    "[project]\nname = \"{name}\"\nkind = \"library\"\n\n[exports]\nunits = [\"{unit}\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n"
                ),
            ),
            file(
                &format!("src/{}.fpas", source_stem(unit)),
                format!(
                    "unit {unit};\n\npublic function Message(): string;\nbegin\n  return 'Hello from {name}'\nend;\n"
                ),
            ),
        ],
    }
}

/// Returns a workspace with one program and one consumed library member.
pub(super) fn workspace(name: &str, identifier: &str) -> TemplatePlan {
    let app_name = format!("{name}-app");
    let library_name = format!("{name}-core");
    let unit = format!("{identifier}.Core");
    TemplatePlan {
        manifest: PathBuf::from(format!("{name}.fpasworkspace")),
        files: vec![
            file(".gitignore", gitignore()),
            file(
                &format!("{name}.fpasworkspace"),
                format!(
                    "[workspace]\nname = \"{name}\"\nmembers = [\n  \"libs/{library_name}/{library_name}.fpasprj\",\n  \"apps/{name}/{app_name}.fpasprj\",\n]\n"
                ),
            ),
            file(
                &format!("libs/{library_name}/{library_name}.fpasprj"),
                format!(
                    "[project]\nname = \"{library_name}\"\nkind = \"library\"\n\n[exports]\nunits = [\"{unit}\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n"
                ),
            ),
            file(
                &format!("libs/{library_name}/src/core.fpas"),
                format!(
                    "unit {unit};\n\npublic function Message(): string;\nbegin\n  return 'Hello from {name}'\nend;\n"
                ),
            ),
            file(
                &format!("apps/{name}/{app_name}.fpasprj"),
                format!(
                    "[project]\nname = \"{app_name}\"\nkind = \"program\"\nmain = \"src/main.fpas\"\n\n[dependencies]\nworkspace = [\"{library_name}\"]\n\n[sources]\ninclude = [\"src/**/*.fpas\"]\n"
                ),
            ),
            file(
                &format!("apps/{name}/src/main.fpas"),
                format!(
                    "program {identifier};\n\nuses {unit}, Std.Console;\n\nbegin\n  WriteLn(Message())\nend.\n"
                ),
            ),
        ],
    }
}

fn source_stem(unit: &str) -> String {
    unit.rsplit('.').next().unwrap_or(unit).to_ascii_lowercase()
}

fn gitignore() -> String {
    "*.fpascp\n*.fpascp.lock\n*.fpascu\n*.fpascu.lock\n".to_string()
}

fn file(path: &str, content: String) -> TemplateFile {
    TemplateFile {
        relative_path: PathBuf::from(path),
        content,
    }
}
