use anyhow::{Context, Result};
use clap::Parser;
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚 ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", "");
static PAPER: Emoji<'_, '_> = Emoji("📄 ", "");
static PACKAGE: Emoji<'_, '_> = Emoji("📦 ", "");

const DEFAULT_TEMPLATE_REPO: &str = "https://github.com/thesandybridge/app-template.git";

#[derive(Parser, Debug)]
#[command(name = "create-sandybridge-app")]
#[command(about = "Scaffold a new sandybridge.io project", long_about = None)]
struct Args {
    /// Name of the project to create
    #[arg(index = 1)]
    name: Option<String>,

    /// Include MDX documentation system
    #[arg(long)]
    docs: bool,

    /// Skip next-auth authentication setup
    #[arg(long)]
    no_auth: bool,

    /// Database type: postgres or none
    #[arg(long, default_value = "postgres")]
    db: String,

    /// Output directory (default: ./<project-name>)
    #[arg(long)]
    path: Option<PathBuf>,

    /// Skip git initialization
    #[arg(long)]
    no_git: bool,

    /// Skip npm install
    #[arg(long)]
    no_install: bool,

    /// Custom template repository URL
    #[arg(long, default_value = DEFAULT_TEMPLATE_REPO)]
    template: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Get project name interactively if not provided
    let project_name = match args.name {
        Some(name) => name,
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Project name")
            .interact_text()?,
    };

    // Validate project name
    let name_regex = Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$")?;
    if !name_regex.is_match(&project_name) {
        anyhow::bail!(
            "Invalid project name '{}'. Use lowercase letters, numbers, and hyphens only.",
            project_name
        );
    }

    // Determine output path
    let output_path = args.path.unwrap_or_else(|| PathBuf::from(&project_name));

    // Check if directory already exists
    if output_path.exists() {
        anyhow::bail!("Directory '{}' already exists", output_path.display());
    }

    // Ask about features if not specified via flags
    let include_docs = args.docs
        || Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Include documentation system (MDX)?")
            .default(false)
            .interact()?;

    let include_auth = !args.no_auth;
    let include_db = args.db != "none";

    println!();
    println!(
        "{} {}Creating {}...",
        style("[1/4]").bold().dim(),
        LOOKING_GLASS,
        style(&project_name).cyan()
    );

    // Clone template repository
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Fetching template...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let status = Command::new("git")
        .args(["clone", "--depth", "1", &args.template, temp_path.to_str().unwrap()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("Failed to clone template repository")?;

    pb.finish_and_clear();

    if !status.success() {
        anyhow::bail!("Failed to clone template from {}", args.template);
    }

    // Remove .git from template
    let template_git = temp_path.join(".git");
    if template_git.exists() {
        fs::remove_dir_all(&template_git)?;
    }

    println!(
        "{} {}Processing template...",
        style("[2/4]").bold().dim(),
        TRUCK
    );

    // Create output directory
    fs::create_dir_all(&output_path)?;

    // Process and copy files
    let files_to_skip = get_files_to_skip(include_docs, include_auth, include_db);

    for entry in WalkDir::new(temp_path).min_depth(1) {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(temp_path)?;
        let relative_str = relative_path.to_string_lossy();

        // Skip files based on features
        if should_skip_file(&relative_str, &files_to_skip) {
            continue;
        }

        // Skip template.json
        if relative_str == "template.json" {
            continue;
        }

        let dest_path = output_path.join(relative_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest_path)?;
        } else {
            // Process file content
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            let processed = process_template(&content, &project_name);

            // Ensure parent directory exists
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&dest_path, processed)?;
        }
    }

    // Update package.json with additional dependencies based on features
    update_package_json(&output_path, include_docs, include_auth, include_db)?;

    println!(
        "{} {}Finalizing project...",
        style("[3/4]").bold().dim(),
        PAPER
    );

    // Initialize git
    if !args.no_git {
        Command::new("git")
            .args(["init"])
            .current_dir(&output_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()?;

        Command::new("git")
            .args(["add", "."])
            .current_dir(&output_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()?;

        Command::new("git")
            .args(["commit", "-m", "Initial commit from create-sandybridge-app"])
            .current_dir(&output_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()?;
    }

    // Install dependencies
    if !args.no_install {
        println!(
            "{} {}Installing dependencies...",
            style("[4/4]").bold().dim(),
            PACKAGE
        );

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Running npm install...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let status = Command::new("npm")
            .args(["install"])
            .current_dir(&output_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        pb.finish_and_clear();

        if let Err(e) = status {
            eprintln!("Warning: Failed to run npm install: {}", e);
            eprintln!("You can run it manually with: cd {} && npm install", output_path.display());
        }
    }

    println!();
    println!("{} Done!", SPARKLE);
    println!();
    println!("  Created {} at {}", style(&project_name).cyan(), style(output_path.display()).green());
    println!();
    println!("  Next steps:");
    println!("    cd {}", project_name);
    if args.no_install {
        println!("    npm install");
    }
    println!("    npm run dev");
    println!();

    Ok(())
}

fn get_files_to_skip(include_docs: bool, include_auth: bool, include_db: bool) -> Vec<&'static str> {
    let mut skip = Vec::new();

    if !include_docs {
        skip.push("app/docs");
        skip.push("content/docs");
        skip.push("lib/docs.ts");
    }

    if !include_auth {
        skip.push("auth.ts");
        skip.push("auth.config.ts");
        skip.push("app/api/auth");
        skip.push("components/session-provider.tsx");
        skip.push("types/next-auth.d.ts");
    }

    if !include_db {
        skip.push("lib/db.ts");
    }

    skip
}

fn should_skip_file(path: &str, skip_patterns: &[&str]) -> bool {
    for pattern in skip_patterns {
        if path.starts_with(pattern) || path == *pattern {
            return true;
        }
    }
    false
}

fn process_template(content: &str, project_name: &str) -> String {
    content.replace("{{project_name}}", project_name)
}

fn update_package_json(
    output_path: &Path,
    include_docs: bool,
    include_auth: bool,
    include_db: bool,
) -> Result<()> {
    let package_json_path = output_path.join("package.json");
    let content = fs::read_to_string(&package_json_path)?;
    let mut package: serde_json::Value = serde_json::from_str(&content)?;

    let deps = package["dependencies"]
        .as_object_mut()
        .context("Invalid package.json")?;

    // Add docs dependencies
    if include_docs {
        deps.insert("@thesandybridge/ui".to_string(), serde_json::json!("^1.0.0"));
        deps.insert("gray-matter".to_string(), serde_json::json!("^4.0.3"));
    }

    // Add auth dependencies
    if include_auth {
        deps.insert("next-auth".to_string(), serde_json::json!("^5.0.0-beta.25"));
        deps.insert("@auth/core".to_string(), serde_json::json!("^0.37.4"));
        deps.insert("jose".to_string(), serde_json::json!("^6.0.11"));
    }

    // Add db dependencies
    if include_db {
        deps.insert("pg".to_string(), serde_json::json!("^8.14.1"));

        let dev_deps = package["devDependencies"]
            .as_object_mut()
            .context("Invalid package.json devDependencies")?;
        dev_deps.insert("@types/pg".to_string(), serde_json::json!("^8.11.11"));
    }

    let formatted = serde_json::to_string_pretty(&package)?;
    fs::write(&package_json_path, formatted)?;

    Ok(())
}
