use anyhow::Result;
use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::manifest::Manifest;
use crate::{
    eval::EvalCase, knowledge::KnowledgeSources, permission::PermissionPolicy, ui::UiLayout,
    workflow::Workflow,
};

/// Generate JSON Schema for Manifest (used for validation).
pub fn manifest_schema() -> Schema {
    let mut gen = SchemaGenerator::default();
    Manifest::json_schema(&mut gen)
}

/// Validate a scenario pack manifest string.
pub fn validate_manifest_yaml(yaml: &str) -> Result<Vec<String>> {
    let manifest = match serde_yaml::from_str::<Manifest>(yaml) {
        Ok(m) => m,
        Err(e) => {
            // Return the parse error as a validation error rather than propagating
            return Ok(vec![format!("manifest.yaml: {}", e)]);
        }
    };
    let mut errors = Vec::new();

    if manifest.id.is_empty() {
        errors.push("manifest.id is required".to_string());
    }
    if manifest.name.is_empty() {
        errors.push("manifest.name is required".to_string());
    }
    if manifest.version.is_empty() {
        errors.push("manifest.version is required".to_string());
    }

    Ok(errors)
}

/// Validate a full scenario pack directory structure.
pub fn validate_pack(dir: &std::path::Path) -> Result<Vec<String>> {
    let mut errors = Vec::new();

    // Check manifest.yaml exists
    let manifest_path = dir.join("manifest.yaml");
    if !manifest_path.exists() {
        errors.push("manifest.yaml is required".to_string());
        return Ok(errors);
    }

    // Validate manifest content
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest = match serde_yaml::from_str::<Manifest>(&content) {
        Ok(manifest) => {
            // Manifest parsed, run field-level checks
            let mut field_errors = validate_manifest_yaml(&content)
                .unwrap_or_else(|e| vec![format!("manifest.yaml: {}", e)]);
            errors.append(&mut field_errors);
            Some(manifest)
        }
        Err(e) => {
            // Parse error - report as validation error instead of failing the whole validation
            errors.push(format!("manifest.yaml: {}", e));
            None
        }
    };

    // Check role.md exists
    let role_path = dir.join("role.md");
    if !role_path.exists() {
        errors.push("role.md is required".to_string());
    }

    if let Some(manifest) = manifest {
        if let Some(entry) = &manifest.entry {
            let entry_path = dir.join(entry);
            if !entry_path.exists() {
                errors.push(format!(
                    "{} is referenced by manifest.entry but does not exist",
                    entry
                ));
            } else {
                validate_yaml_file::<Workflow>(&entry_path, entry, &mut errors);
                validate_workflow_step_types(&entry_path, entry, &mut errors);
            }
        }
    }

    validate_yaml_group_with_steps::<Workflow>(
        dir,
        "workflows",
        &["yaml", "yml"],
        "workflow",
        &mut errors,
    );
    validate_yaml_group::<serde_yaml::Value>(
        dir,
        "tools",
        &["yaml", "yml"],
        "tool config",
        &mut errors,
    );
    validate_optional_yaml::<KnowledgeSources>(dir, "knowledge/sources.yaml", &mut errors);
    validate_optional_yaml::<UiLayout>(dir, "ui/layout.yaml", &mut errors);
    validate_optional_yaml::<PermissionPolicy>(dir, "permissions/policy.yaml", &mut errors);
    validate_optional_yaml::<Vec<EvalCase>>(dir, "evals/cases.yaml", &mut errors);
    dedup_errors(&mut errors);

    Ok(errors)
}

fn validate_yaml_file<T>(path: &std::path::Path, label: &str, errors: &mut Vec<String>)
where
    T: DeserializeOwned,
{
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if let Err(e) = serde_yaml::from_str::<T>(&content) {
                errors.push(format!("{}: {}", label, e));
            }
        }
        Err(e) => errors.push(format!("{}: {}", label, e)),
    }
}

fn validate_optional_yaml<T>(dir: &std::path::Path, relative: &str, errors: &mut Vec<String>)
where
    T: DeserializeOwned,
{
    let path = dir.join(relative);
    if path.exists() {
        validate_yaml_file::<T>(&path, relative, errors);
    }
}

fn validate_yaml_group<T>(
    dir: &std::path::Path,
    relative_dir: &str,
    extensions: &[&str],
    kind: &str,
    errors: &mut Vec<String>,
) where
    T: DeserializeOwned,
{
    let group_dir = dir.join(relative_dir);
    if !group_dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(&group_dir) {
        Ok(entries) => entries,
        Err(e) => {
            errors.push(format!("{}: {}", relative_dir, e));
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                errors.push(format!("{}: {}", relative_dir, e));
                continue;
            }
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if !extensions.contains(&extension) {
            continue;
        }

        let label = path
            .strip_prefix(dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| format!("{} file", kind));
        validate_yaml_file::<T>(&path, &label, errors);
    }
}

/// Allowed workflow step type prefixes.
/// Valid patterns: "tool", "tool.*", "agent.*", "ui.*", "governance.*"
fn is_valid_step_type(step_type: &str) -> bool {
    if step_type == "tool" {
        return true;
    }
    step_type.starts_with("tool.")
        || step_type.starts_with("agent.")
        || step_type.starts_with("ui.")
        || step_type.starts_with("governance.")
}

fn validate_workflow_step_types(path: &std::path::Path, label: &str, errors: &mut Vec<String>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("{}: {}", label, e));
            return;
        }
    };
    let workflow = match serde_yaml::from_str::<Workflow>(&content) {
        Ok(w) => w,
        _ => return, // YAML parse error already reported by validate_yaml_file
    };
    for step in &workflow.steps {
        if !is_valid_step_type(&step.step_type) {
            errors.push(format!(
                "{}: step '{}' has unknown type '{}'. Valid types: tool, tool.*, agent.*, ui.*",
                label, step.id, step.step_type
            ));
        }
    }
}

fn dedup_errors(errors: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    errors.retain(|error| seen.insert(error.clone()));
}

fn validate_yaml_group_with_steps<T>(
    dir: &std::path::Path,
    relative_dir: &str,
    extensions: &[&str],
    kind: &str,
    errors: &mut Vec<String>,
) where
    T: DeserializeOwned,
{
    let group_dir = dir.join(relative_dir);
    if !group_dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(&group_dir) {
        Ok(entries) => entries,
        Err(e) => {
            errors.push(format!("{}: {}", relative_dir, e));
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                errors.push(format!("{}: {}", relative_dir, e));
                continue;
            }
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if !extensions.contains(&extension) {
            continue;
        }

        let label = path
            .strip_prefix(dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| format!("{} file", kind));
        validate_yaml_file::<T>(&path, &label, errors);
        // Also validate workflow step types for YAML files in workflows/
        if let Ok(_) =
            serde_yaml::from_str::<Workflow>(&std::fs::read_to_string(&path).unwrap_or_default())
        {
            validate_workflow_step_types(&path, &label, errors);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("melon-test-{}", uuid::Uuid::new_v4()))
    }

    fn write_file(dir: &std::path::Path, path: &str, content: &str) {
        let full = dir.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, content).unwrap();
    }

    fn valid_manifest() -> &'static str {
        "id: test.pack\nname: Test Pack\nversion: 0.1.0\n"
    }

    fn valid_pack_dir() -> std::path::PathBuf {
        let dir = temp_dir();
        write_file(&dir, "manifest.yaml", valid_manifest());
        write_file(&dir, "role.md", "# Test Role\n");
        write_file(
            &dir,
            "workflows/default.yaml",
            "name: test\nsteps:\n  - id: step1\n    type: tool\n    action: mock_tool\n",
        );
        dir
    }

    #[test]
    fn valid_manifest_has_no_errors() {
        let errors = validate_manifest_yaml(valid_manifest()).unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn missing_id_produces_error() {
        let yaml = "name: Test Pack\nversion: 0.1.0\n";
        let errors = validate_manifest_yaml(yaml).unwrap();
        assert!(!errors.is_empty(), "missing id should produce errors");
        assert!(errors.iter().any(|e| e.contains("id")));
    }

    #[test]
    fn missing_name_produces_error() {
        let yaml = "id: test.pack\nversion: 0.1.0\n";
        let errors = validate_manifest_yaml(yaml).unwrap();
        assert!(!errors.is_empty(), "missing name should produce errors");
        assert!(errors.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn missing_version_produces_error() {
        let yaml = "id: test.pack\nname: Test Pack\n";
        let errors = validate_manifest_yaml(yaml).unwrap();
        assert!(!errors.is_empty(), "missing version should produce errors");
        assert!(errors.iter().any(|e| e.contains("version")));
    }

    #[test]
    fn valid_pack_passes_validation() {
        let dir = valid_pack_dir();
        let errors = validate_pack(&dir).unwrap();
        assert!(errors.is_empty(), "unexpected errors: {:?}", errors);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_manifest_fails() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let errors = validate_pack(&dir).unwrap();
        assert!(errors
            .iter()
            .any(|e| e.contains("manifest.yaml is required")));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_role_fails() {
        let dir = temp_dir();
        write_file(&dir, "manifest.yaml", valid_manifest());
        let errors = validate_pack(&dir).unwrap();
        assert!(errors.iter().any(|e| e.contains("role.md is required")));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_entry_workflow_fails() {
        let dir = temp_dir();
        write_file(
            &dir,
            "manifest.yaml",
            "id: test.pack\nname: Test Pack\nversion: 0.1.0\nentry: workflows/missing.yaml\n",
        );
        write_file(&dir, "role.md", "# Role\n");
        let errors = validate_pack(&dir).unwrap();
        assert!(errors.iter().any(|e| e.contains("workflows/missing.yaml")));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn invalid_workflow_fails() {
        let dir = temp_dir();
        write_file(&dir, "manifest.yaml", valid_manifest());
        write_file(&dir, "role.md", "# Role\n");
        write_file(&dir, "workflows/default.yaml", "not: valid: [workflow\n");
        let errors = validate_pack(&dir).unwrap();
        assert!(!errors.is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn invalid_permission_policy_fails() {
        let dir = valid_pack_dir();
        write_file(
            &dir,
            "permissions/policy.yaml",
            "policies:\n  bad: invalid\n",
        );
        let errors = validate_pack(&dir).unwrap();
        assert!(errors.iter().any(|e| e.contains("permissions/policy.yaml")));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn optional_files_do_not_cause_errors_when_missing() {
        let dir = valid_pack_dir();
        let errors = validate_pack(&dir).unwrap();
        assert!(
            errors.is_empty(),
            "optional files should not cause errors: {:?}",
            errors
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn real_demo_ops_pack_validates() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace = manifest_dir.parent().unwrap().parent().unwrap();
        let pack_dir = workspace.join("scenarios/demo-ops");
        if !pack_dir.exists() {
            return; // skip if demo pack not present
        }
        let errors = validate_pack(&pack_dir).unwrap();
        assert!(
            errors.is_empty(),
            "demo-ops pack should validate: {:?}",
            errors
        );
    }

    #[test]
    fn real_melon_home_pack_validates() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace = manifest_dir.parent().unwrap().parent().unwrap();
        let pack_dir = workspace.join("scenarios/melon-home");
        if !pack_dir.exists() {
            return; // skip if melon-home pack not present
        }
        let errors = validate_pack(&pack_dir).unwrap();
        assert!(
            errors.is_empty(),
            "melon-home pack should validate: {:?}",
            errors
        );
    }

    #[test]
    fn invalid_step_type_fails() {
        let dir = temp_dir();
        write_file(&dir, "manifest.yaml", valid_manifest());
        write_file(&dir, "role.md", "# Role\n");
        write_file(
            &dir,
            "workflows/default.yaml",
            "name: test\nsteps:\n  - id: step1\n    type: invalid_type\n",
        );
        let errors = validate_pack(&dir).unwrap();
        assert!(errors
            .iter()
            .any(|e| e.contains("unknown type") || e.contains("invalid_type")));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_manifest_field_returns_validation_error() {
        let dir = temp_dir();
        // Write manifest missing required fields - serde_yaml will fail to parse
        write_file(&dir, "manifest.yaml", "version: 0.1.0\n");
        write_file(&dir, "role.md", "# Role\n");
        // validate_pack should NOT return Err, it should return validation errors
        let result = validate_pack(&dir);
        assert!(
            result.is_ok(),
            "validate_pack should not fail on missing fields"
        );
        let errors = result.unwrap();
        assert!(!errors.is_empty());
        fs::remove_dir_all(&dir).ok();
    }
}
