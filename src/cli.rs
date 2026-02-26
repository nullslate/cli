use clap::{Parser, Subcommand};
use std::path::PathBuf;

const DEFAULT_TEMPLATE_REPO: &str = "https://github.com/nullslate/app-template.git";

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
    #[arg(long, default_value = DEFAULT_TEMPLATE_REPO)]
    pub template: String,

    /// Accept all defaults without prompting
    #[arg(short, long)]
    pub yes: bool,
}
