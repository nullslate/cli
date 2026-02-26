use clap::{Parser, Subcommand};
use std::path::PathBuf;

const DEFAULT_TEMPLATE_REPO: &str = "https://github.com/nullslate/app-template.git";
const DEFAULT_LIB_TEMPLATE_REPO: &str = "https://github.com/nullslate/lib-template.git";
const DEFAULT_FULLSTACK_TEMPLATE_REPO: &str = "https://github.com/nullslate/app-template.git";

pub fn default_template_url(project_type: &str) -> &'static str {
    match project_type {
        "lib" => DEFAULT_LIB_TEMPLATE_REPO,
        "fullstack" => DEFAULT_FULLSTACK_TEMPLATE_REPO,
        _ => DEFAULT_TEMPLATE_REPO,
    }
}

#[derive(Parser, Debug)]
#[command(name = "nullslate")]
#[command(about = "CLI for the nullslate dev tooling ecosystem", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scaffold a new project
    Init(InitArgs),
    /// Start the dev environment (auto-detects project type)
    Dev,
    /// Build the project (auto-detects project type)
    Build,
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Name of the project to create
    #[arg(index = 1)]
    pub name: Option<String>,

    /// Include MDX documentation system
    #[arg(long)]
    pub docs: bool,

    /// Skip Auth.js authentication setup
    #[arg(long)]
    pub no_auth: bool,

    /// Database type: postgres or none
    #[arg(long, default_value = "none")]
    pub db: String,

    /// Output directory (default: ./<project-name>)
    #[arg(long)]
    pub path: Option<PathBuf>,

    /// Skip git initialization
    #[arg(long)]
    pub no_git: bool,

    /// Skip npm install
    #[arg(long)]
    pub no_install: bool,

    /// Custom template repository URL
    #[arg(long)]
    pub template: Option<String>,

    /// Project type: app or lib
    #[arg(long, default_value = "app")]
    pub project_type: String,

    /// Shorthand for --project-type lib
    #[arg(long)]
    pub lib: bool,

    /// Shorthand for --project-type fullstack
    #[arg(long)]
    pub fullstack: bool,

    /// Language: typescript or javascript (lib only)
    #[arg(long, default_value = "typescript")]
    pub lang: String,

    /// Include React support (lib only)
    #[arg(long)]
    pub react: bool,

    /// Include Tailwind CSS (lib only)
    #[arg(long)]
    pub css: bool,

    /// Include Vitest testing (lib only)
    #[arg(long)]
    pub testing: bool,

    /// Accept all defaults without prompting
    #[arg(short, long)]
    pub yes: bool,
}
