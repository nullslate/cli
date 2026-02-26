use anyhow::Result;

pub fn intro() -> Result<()> {
    cliclack::clear_screen()?;
    cliclack::intro("nullslate")?;
    Ok(())
}

pub fn prompt_project_name() -> Result<String> {
    let name: String = cliclack::input("Project name")
        .placeholder("my-app")
        .validate(|input: &String| {
            let re = regex::Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$|^[a-z0-9]$").unwrap();
            if re.is_match(input) {
                Ok(())
            } else {
                Err("Use lowercase letters, numbers, and hyphens only")
            }
        })
        .interact()?;
    Ok(name)
}

#[derive(Clone, PartialEq, Eq)]
pub enum Feature {
    Auth,
    Docs,
    Db,
}

pub fn prompt_features() -> Result<Vec<Feature>> {
    let features: Vec<Feature> = cliclack::multiselect("Select features")
        .item(Feature::Auth, "Authentication", "Auth.js with GitHub OAuth")
        .item(Feature::Docs, "Documentation", "MDX docs system")
        .item(Feature::Db, "Database", "PostgreSQL")
        .initial_values(vec![Feature::Auth])
        .required(false)
        .interact()?;
    Ok(features)
}

pub fn create_spinner(message: &str) -> cliclack::ProgressBar {
    let spinner = cliclack::spinner();
    spinner.start(message);
    spinner
}

pub fn outro_success(project_name: &str, output_path: &std::path::Path, no_install: bool) {
    let mut next_steps = format!("cd {}", project_name);
    if no_install {
        next_steps.push_str("\n    bun install");
    }
    next_steps.push_str("\n    bun dev");

    let msg = format!(
        "Created {} at {}\n\n  Next steps:\n    {}",
        project_name,
        output_path.display(),
        next_steps
    );
    let _ = cliclack::outro(msg);
}

pub fn outro_success_fullstack(project_name: &str, output_path: &std::path::Path) {
    let msg = format!(
        "Created {} at {}\n\n  Next steps:\n    cd {}\n    ns dev",
        project_name,
        output_path.display(),
        project_name,
    );
    let _ = cliclack::outro(msg);
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProjectType {
    App,
    Fullstack,
    Lib,
}

pub fn prompt_project_type() -> Result<ProjectType> {
    let project_type: ProjectType = cliclack::select("What are you building?")
        .item(ProjectType::App, "Application", "Full-stack TanStack Start app")
        .item(ProjectType::Fullstack, "Fullstack", "Rust API + TanStack Start frontend with devforge")
        .item(ProjectType::Lib, "Library", "Publishable package (Vite lib mode + tsup)")
        .interact()?;
    Ok(project_type)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LibFeature {
    React,
    Css,
    Testing,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Language {
    TypeScript,
    JavaScript,
}

pub fn prompt_language() -> Result<Language> {
    let lang: Language = cliclack::select("Language")
        .item(Language::TypeScript, "TypeScript", "Recommended")
        .item(Language::JavaScript, "JavaScript", "Plain JS, no type checking")
        .interact()?;
    Ok(lang)
}

pub fn prompt_lib_features() -> Result<Vec<LibFeature>> {
    let features: Vec<LibFeature> = cliclack::multiselect("Select features")
        .item(LibFeature::React, "React", "JSX support, React peer dependency")
        .item(LibFeature::Css, "CSS/Tailwind", "Tailwind CSS styling")
        .item(LibFeature::Testing, "Testing", "Vitest test setup")
        .required(false)
        .interact()?;
    Ok(features)
}

pub fn outro_success_lib(project_name: &str, output_path: &std::path::Path, no_install: bool) {
    let mut next_steps = format!("cd {}", project_name);
    if no_install {
        next_steps.push_str("\n    bun install");
    }
    next_steps.push_str("\n    bun run build");

    let msg = format!(
        "Created {} at {}\n\n  Next steps:\n    {}",
        project_name,
        output_path.display(),
        next_steps
    );
    let _ = cliclack::outro(msg);
}

pub fn outro_cancel(message: &str) {
    let _ = cliclack::outro_cancel(message);
}
