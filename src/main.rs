mod cli;
mod features;
mod template;
mod ui;

use anyhow::Result;
use clap::Parser;
use regex::Regex;
use std::path::PathBuf;

use cli::{Cli, Commands, InitArgs, default_template_url};
use features::{
    cleanup_layout_for_no_auth, generate_env_file, get_files_to_skip, get_lib_files_to_skip,
    update_lib_package_json, update_package_json,
};
use template::{clone_template, copy_template, init_git, install_deps};
use ui::{
    create_spinner, outro_cancel, outro_success, outro_success_lib, Feature, Language, LibFeature,
    ProjectType,
};

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
        Some(ref name) => name.clone(),
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

    let output_path = args.path.clone().unwrap_or_else(|| PathBuf::from(&project_name));

    if output_path.exists() {
        let msg = format!("Directory '{}' already exists", output_path.display());
        outro_cancel(&msg);
        anyhow::bail!("{}", msg);
    }

    // Determine project type
    let is_lib = if args.yes {
        args.lib || args.project_type == "lib"
    } else {
        let pt = ui::prompt_project_type()?;
        pt == ProjectType::Lib
    };

    let template_url = args
        .template
        .clone()
        .unwrap_or_else(|| default_template_url(is_lib).to_string());

    // Clone template
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path();

    let spinner = create_spinner("Fetching template...");
    clone_template(&template_url, temp_path)?;
    spinner.stop("Template fetched");

    if is_lib {
        cmd_init_lib(&args, &project_name, &output_path, temp_path)?;
    } else {
        cmd_init_app(&args, &project_name, &output_path, temp_path)?;
    }

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

    if is_lib {
        outro_success_lib(&project_name, &output_path, args.no_install);
    } else {
        outro_success(&project_name, &output_path, args.no_install);
    }

    Ok(())
}

fn cmd_init_app(
    args: &InitArgs,
    project_name: &str,
    output_path: &PathBuf,
    temp_path: &std::path::Path,
) -> Result<()> {
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

    let spinner = create_spinner("Processing files...");

    let files_to_skip = get_files_to_skip(include_docs, include_auth, include_db);
    copy_template(temp_path, output_path, project_name, &files_to_skip)?;
    update_package_json(output_path, include_docs, include_auth, include_db)?;

    if !include_auth {
        cleanup_layout_for_no_auth(output_path)?;
    }

    if include_auth {
        generate_env_file(output_path)?;
    }

    spinner.stop("Files processed");
    Ok(())
}

fn cmd_init_lib(
    args: &InitArgs,
    project_name: &str,
    output_path: &PathBuf,
    temp_path: &std::path::Path,
) -> Result<()> {
    let (lang, include_react, include_css, include_testing) = if args.yes {
        (
            args.lang.clone(),
            args.react,
            args.css,
            args.testing,
        )
    } else {
        let language = ui::prompt_language()?;
        let selected = ui::prompt_lib_features()?;
        (
            match language {
                Language::TypeScript => "typescript".to_string(),
                Language::JavaScript => "javascript".to_string(),
            },
            selected.contains(&LibFeature::React),
            selected.contains(&LibFeature::Css),
            selected.contains(&LibFeature::Testing),
        )
    };

    let spinner = create_spinner("Processing files...");

    let files_to_skip =
        get_lib_files_to_skip(&lang, include_react, include_css, include_testing);
    copy_template(temp_path, output_path, project_name, &files_to_skip)?;
    update_lib_package_json(output_path, &lang, include_react, include_css, include_testing)?;

    spinner.stop("Files processed");
    Ok(())
}
