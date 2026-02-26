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

pub fn outro_cancel(message: &str) {
    let _ = cliclack::outro_cancel(message);
}
