use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq)]
pub(crate) enum ProjectKind {
    Fullstack,
    Frontend,
    Rust,
}

/// Walk up from CWD to find project root and its type.
pub(crate) fn detect_project(start: &Path) -> Result<(PathBuf, ProjectKind)> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join("devforge.toml").exists() {
            return Ok((dir, ProjectKind::Fullstack));
        }
        if dir.join("package.json").exists() {
            return Ok((dir, ProjectKind::Frontend));
        }
        if dir.join("Cargo.toml").exists() {
            return Ok((dir, ProjectKind::Rust));
        }
        if !dir.pop() {
            anyhow::bail!(
                "No project root found (looked for devforge.toml, package.json, or Cargo.toml)"
            );
        }
    }
}

pub fn cmd_dev() -> Result<()> {
    let cwd = env::current_dir().context("failed to get current directory")?;
    let (root, kind) = detect_project(&cwd)?;

    match kind {
        ProjectKind::Fullstack => {
            run_cmd(&root, "cargo", &["xtask", "dev"])?;
        }
        ProjectKind::Frontend => {
            run_cmd(&root, "bun", &["dev"])?;
        }
        ProjectKind::Rust => {
            run_cmd(&root, "cargo", &["run"])?;
        }
    }
    Ok(())
}

pub fn cmd_build() -> Result<()> {
    let cwd = env::current_dir().context("failed to get current directory")?;
    let (root, kind) = detect_project(&cwd)?;

    match kind {
        ProjectKind::Fullstack => {
            run_cmd(&root, "cargo", &["build", "--release"])?;
            let web_dir = root.join("web");
            if web_dir.exists() {
                run_cmd(&web_dir, "bun", &["run", "build"])?;
            }
        }
        ProjectKind::Frontend => {
            run_cmd(&root, "bun", &["run", "build"])?;
        }
        ProjectKind::Rust => {
            run_cmd(&root, "cargo", &["build", "--release"])?;
        }
    }
    Ok(())
}

fn run_cmd(dir: &Path, program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("failed to run {program}"))?;

    if !status.success() {
        anyhow::bail!("{program} exited with {status}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detect_fullstack_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("devforge.toml"), "").unwrap();
        let (root, kind) = detect_project(dir.path()).unwrap();
        assert_eq!(root, dir.path());
        assert_eq!(kind, ProjectKind::Fullstack);
    }

    #[test]
    fn detect_frontend_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let (root, kind) = detect_project(dir.path()).unwrap();
        assert_eq!(root, dir.path());
        assert_eq!(kind, ProjectKind::Frontend);
    }

    #[test]
    fn detect_rust_project() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let (root, kind) = detect_project(dir.path()).unwrap();
        assert_eq!(root, dir.path());
        assert_eq!(kind, ProjectKind::Rust);
    }

    #[test]
    fn detect_fullstack_takes_priority() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("devforge.toml"), "").unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        let (_, kind) = detect_project(dir.path()).unwrap();
        assert_eq!(kind, ProjectKind::Fullstack);
    }

    #[test]
    fn detect_walks_up() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        let sub = dir.path().join("src");
        fs::create_dir(&sub).unwrap();
        let (root, kind) = detect_project(&sub).unwrap();
        assert_eq!(root, dir.path().to_path_buf());
        assert_eq!(kind, ProjectKind::Frontend);
    }

    #[test]
    fn detect_no_project_fails() {
        let dir = tempdir().unwrap();
        let result = detect_project(dir.path());
        assert!(result.is_err());
    }
}
