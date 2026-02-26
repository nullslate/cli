use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::features::{should_skip_file, get_files_to_skip, update_package_json, cleanup_layout_for_no_auth, generate_env_file};
use crate::template::process_template;

const FULLSTACK_SUBDIR: &str = "fullstack";

/// Scaffold a fullstack project:
/// 1. Copy frontend files to {output}/web/
/// 2. Copy fullstack overlay files to {output}/
pub fn scaffold_fullstack(
    temp_path: &Path,
    output_path: &Path,
    project_name: &str,
    include_auth: bool,
    include_docs: bool,
) -> Result<()> {
    fs::create_dir_all(output_path)?;

    let web_path = output_path.join("web");

    // Step 1: Copy frontend files to web/
    let files_to_skip = get_files_to_skip(include_docs, include_auth, false);
    copy_filtered(temp_path, &web_path, project_name, &files_to_skip, &[FULLSTACK_SUBDIR, "template.json"])?;

    // Update web/package.json (remove deps for disabled features)
    update_package_json(&web_path, include_docs, include_auth, false)?;

    if !include_auth {
        cleanup_layout_for_no_auth(&web_path)?;
    }

    // Step 2: Copy fullstack overlay files to root
    let fullstack_src = temp_path.join(FULLSTACK_SUBDIR);
    if fullstack_src.is_dir() {
        copy_filtered(&fullstack_src, output_path, project_name, &[], &["template.json"])?;
    }

    // Step 3: Generate .env for auth in web/ if needed
    if include_auth {
        generate_env_file(&web_path)?;
    }

    Ok(())
}

fn copy_filtered(
    src: &Path,
    dest: &Path,
    project_name: &str,
    files_to_skip: &[&str],
    extra_skip: &[&str],
) -> Result<()> {
    fs::create_dir_all(dest)?;

    for entry in WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(src)?;
        let relative_str = relative_path.to_string_lossy();

        if should_skip_file(&relative_str, files_to_skip) {
            continue;
        }

        // Skip extra patterns (e.g. fullstack/ subdir when copying frontend)
        let skip_extra = extra_skip.iter().any(|p| relative_str.starts_with(p) || relative_str == *p);
        if skip_extra {
            continue;
        }

        let dest_path = dest.join(relative_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
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
