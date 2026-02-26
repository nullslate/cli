mod cli;
mod features;
mod template;
mod ui;

use anyhow::Result;
use clap::Parser;
use regex::Regex;
use std::path::PathBuf;

use cli::{Cli, Commands, InitArgs};
use features::{
    cleanup_layout_for_no_auth, generate_env_file, get_files_to_skip, update_package_json,
};
use template::{clone_template, copy_template, init_git, install_deps};
use ui::{create_spinner, outro_cancel, outro_success, Feature};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => cmd_init(args),
    }
}

fn cmd_init(args: InitArgs) -> Result<()> {
    if !args.yes {
        ui::intro()?;
    }

    let project_name = match args.name {
        Some(name) => name,
        None => {
            if args.yes {
                outro_cancel("Project name is required when using --yes flag");
                anyhow::bail!("Project name is required when using --yes flag");
            }
            ui::prompt_project_name()?
        }
    };

    let name_regex = Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$")?;
    if !name_regex.is_match(&project_name) {
        let msg = format!(
            "Invalid project name '{}'. Use lowercase letters, numbers, and hyphens only.",
            project_name
        );
        outro_cancel(&msg);
        anyhow::bail!("{}", msg);
    }

    let output_path = args.path.unwrap_or_else(|| PathBuf::from(&project_name));

    if output_path.exists() {
        let msg = format!("Directory '{}' already exists", output_path.display());
        outro_cancel(&msg);
        anyhow::bail!("{}", msg);
    }

    let (include_auth, include_docs, include_db) = if args.yes {
        (!args.no_auth, args.docs, args.db != "none")
    } else {
        let selected = ui::prompt_features()?;
        (
            selected.contains(&Feature::Auth),
            selected.contains(&Feature::Docs),
            selected.contains(&Feature::Db),
        )
    };

    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    let spinner = create_spinner("Fetching template...");
    clone_template(&args.template, temp_path)?;
    spinner.stop("Template fetched");

    let spinner = create_spinner("Processing files...");

    let files_to_skip = get_files_to_skip(include_docs, include_auth, include_db);
    copy_template(temp_path, &output_path, &project_name, &files_to_skip)?;

    update_package_json(&output_path, include_docs, include_auth, include_db)?;

    if !include_auth {
        cleanup_layout_for_no_auth(&output_path)?;
    }

    if include_auth {
        generate_env_file(&output_path)?;
    }

    spinner.stop("Files processed");

    if !args.no_git {
        let spinner = create_spinner("Initializing git...");
        init_git(&output_path)?;
        spinner.stop("Git initialized");
    }

    if !args.no_install {
        let spinner = create_spinner("Installing dependencies...");
        install_deps(&output_path)?;
        spinner.stop("Dependencies installed");
    }

    outro_success(&project_name, &output_path, args.no_install);

    Ok(())
}
