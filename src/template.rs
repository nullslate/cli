use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

use crate::features::should_skip_file;

pub fn clone_template(template_url: &str, dest: &Path) -> Result<()> {
    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            template_url,
            dest.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("Failed to clone template repository")?;

    if !status.success() {
        anyhow::bail!("Failed to clone template from {}", template_url);
    }

    // Remove .git from template
    let git_dir = dest.join(".git");
    if git_dir.exists() {
        fs::remove_dir_all(&git_dir)?;
    }

    Ok(())
}

pub fn process_template(content: &str, project_name: &str) -> String {
    content.replace("{{project_name}}", project_name)
}

pub fn copy_template(
    temp_path: &Path,
    output_path: &Path,
    project_name: &str,
    files_to_skip: &[&str],
) -> Result<()> {
    fs::create_dir_all(output_path)?;

    for entry in WalkDir::new(temp_path).min_depth(1) {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(temp_path)?;
        let relative_str = relative_path.to_string_lossy();

        if should_skip_file(&relative_str, files_to_skip) {
            continue;
        }

        if relative_str == "template.json" {
            continue;
        }

        let dest_path = output_path.join(relative_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Try to read as UTF-8 for template processing, fall back to binary copy
            match fs::read_to_string(entry.path()) {
                Ok(content) => {
                    let processed = process_template(&content, project_name);
                    fs::write(&dest_path, processed)?;
                }
                Err(_) => {
                    fs::copy(entry.path(), &dest_path)?;
                }
            }
        }
    }

    Ok(())
}

pub fn init_git(output_path: &Path) -> Result<()> {
    Command::new("git")
        .args(["init"])
        .current_dir(output_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    Command::new("git")
        .args(["add", "."])
        .current_dir(output_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    Command::new("git")
        .args(["commit", "-m", "Initial commit from nullslate"])
        .current_dir(output_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    Ok(())
}

pub fn install_deps(output_path: &Path) -> Result<()> {
    let status = Command::new("npm")
        .args(["install"])
        .current_dir(output_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if let Err(e) = status {
        eprintln!(
            "Warning: Failed to run npm install: {}",
            e
        );
        eprintln!(
            "You can run it manually with: cd {} && npm install",
            output_path.display()
        );
    }

    // Update packages to latest versions
    let update_status = Command::new("npm")
        .args([
            "install",
            "@thesandybridge/themes@latest",
            "@thesandybridge/ui@latest",
        ])
        .current_dir(output_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    if let Err(e) = update_status {
        eprintln!(
            "Warning: Failed to update packages: {}",
            e
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_template_replaces_placeholder() {
        let result = process_template("name: {{project_name}}", "my-app");
        assert_eq!(result, "name: my-app");
    }

    #[test]
    fn process_template_no_placeholder() {
        let input = "no placeholders here";
        let result = process_template(input, "my-app");
        assert_eq!(result, input);
    }

    #[test]
    fn process_template_multiple_placeholders() {
        let result = process_template("{{project_name}} and {{project_name}}", "my-app");
        assert_eq!(result, "my-app and my-app");
    }
}
