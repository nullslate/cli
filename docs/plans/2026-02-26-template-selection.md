# Template Selection Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add project type selection (App vs Library) to the nullslate CLI, with a separate lib-template repo for library scaffolding.

**Architecture:** Add a `ProjectType` enum and branch the init flow based on selection. App path stays unchanged. Lib path uses a new template repo with its own feature set (language, react, css, testing). All template logic remains hardcoded in the CLI per type.

**Tech Stack:** Rust, clap, cliclack, existing template clone/copy infrastructure.

---

### Task 1: Add ProjectType enum and prompt

**Files:**
- Modify: `src/ui.rs`

**Step 1: Add ProjectType enum and prompt function**

Add after the existing `Feature` enum in `src/ui.rs`:

```rust
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProjectType {
    App,
    Lib,
}

pub fn prompt_project_type() -> Result<ProjectType> {
    let project_type: ProjectType = cliclack::select("What are you building?")
        .item(ProjectType::App, "Application", "Full-stack TanStack Start app")
        .item(ProjectType::Lib, "Library", "Publishable package (Vite lib mode + tsup)")
        .interact()?;
    Ok(project_type)
}
```

**Step 2: Add LibFeature enum and prompt**

Add after `prompt_project_type` in `src/ui.rs`:

```rust
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
```

**Step 3: Update outro_success for lib projects**

Add a new function in `src/ui.rs`:

```rust
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
```

**Step 4: Run existing tests to ensure nothing broke**

Run: `cargo test`
Expected: All existing tests pass.

**Step 5: Commit**

```bash
git add src/ui.rs
git commit -m "feat: add project type, language, and lib feature prompts"
```

---

### Task 2: Add CLI flags for lib mode

**Files:**
- Modify: `src/cli.rs`

**Step 1: Add lib-related flags to InitArgs**

Update `src/cli.rs` — add a `DEFAULT_LIB_TEMPLATE_REPO` constant and new fields to `InitArgs`:

```rust
const DEFAULT_LIB_TEMPLATE_REPO: &str = "https://github.com/nullslate/lib-template.git";
```

Add these fields to `InitArgs` (after existing fields, before `yes`):

```rust
    /// Project type: app or lib
    #[arg(long, default_value = "app")]
    pub project_type: String,

    /// Shorthand for --project-type lib
    #[arg(long)]
    pub lib: bool,

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
```

Also make the `template` field use a helper to pick default based on type. For now, keep the default as app template — we'll resolve it at runtime in `main.rs`.

Remove the `default_value` from `template` so we can detect when the user didn't pass it:

```rust
    /// Custom template repository URL
    #[arg(long)]
    pub template: Option<String>,
```

Add a public function to get the default template URL:

```rust
pub fn default_template_url(is_lib: bool) -> &'static str {
    if is_lib {
        DEFAULT_LIB_TEMPLATE_REPO
    } else {
        DEFAULT_TEMPLATE_REPO
    }
}
```

**Step 2: Compile check**

Run: `cargo check`
Expected: Compile errors in `main.rs` because `args.template` is now `Option<String>`. We'll fix that in Task 3.

**Step 3: Commit**

```bash
git add src/cli.rs
git commit -m "feat: add CLI flags for lib project type"
```

---

### Task 3: Add lib feature processing in features.rs

**Files:**
- Modify: `src/features.rs`

**Step 1: Add lib file skip and package.json logic**

Add these functions to `src/features.rs`:

```rust
pub fn get_lib_files_to_skip(
    lang: &str,
    include_react: bool,
    include_css: bool,
    include_testing: bool,
) -> Vec<&'static str> {
    let mut skip = Vec::new();

    if lang == "javascript" {
        skip.push("tsconfig.json");
    }

    if !include_react {
        skip.push("src/components");
    }

    if !include_css {
        skip.push("src/styles");
    }

    if !include_testing {
        skip.push("vitest.config.ts");
        skip.push("src/__tests__");
    }

    skip
}

pub fn update_lib_package_json(
    output_path: &Path,
    lang: &str,
    include_react: bool,
    include_css: bool,
    include_testing: bool,
) -> Result<()> {
    let package_json_path = output_path.join("package.json");
    let content = fs::read_to_string(&package_json_path)?;
    let mut package: serde_json::Value = serde_json::from_str(&content)?;

    if lang == "javascript" {
        if let Some(dev_deps) = package["devDependencies"].as_object_mut() {
            dev_deps.remove("typescript");
        }
    }

    if !include_react {
        if let Some(peer_deps) = package["peerDependencies"].as_object_mut() {
            peer_deps.remove("react");
            peer_deps.remove("react-dom");
        }
        if let Some(dev_deps) = package["devDependencies"].as_object_mut() {
            dev_deps.remove("@types/react");
            dev_deps.remove("@types/react-dom");
            dev_deps.remove("react");
            dev_deps.remove("react-dom");
        }
    }

    if !include_css {
        if let Some(deps) = package["dependencies"].as_object_mut() {
            deps.remove("tailwindcss");
            deps.remove("@tailwindcss/vite");
        }
        if let Some(dev_deps) = package["devDependencies"].as_object_mut() {
            dev_deps.remove("tailwindcss");
            dev_deps.remove("@tailwindcss/vite");
        }
    }

    if !include_testing {
        if let Some(dev_deps) = package["devDependencies"].as_object_mut() {
            dev_deps.remove("vitest");
            dev_deps.remove("@testing-library/react");
            dev_deps.remove("@testing-library/jest-dom");
            dev_deps.remove("jsdom");
        }
        // Remove test script
        if let Some(scripts) = package["scripts"].as_object_mut() {
            scripts.remove("test");
        }
    }

    let formatted = serde_json::to_string_pretty(&package)?;
    fs::write(&package_json_path, formatted)?;

    Ok(())
}
```

**Step 2: Add unit tests for lib skip logic**

Add to the `#[cfg(test)]` module in `src/features.rs`:

```rust
    #[test]
    fn get_lib_files_to_skip_no_features() {
        let skip = get_lib_files_to_skip("typescript", false, false, false);
        assert!(skip.contains(&"src/components"));
        assert!(skip.contains(&"src/styles"));
        assert!(skip.contains(&"vitest.config.ts"));
        assert!(!skip.contains(&"tsconfig.json"));
    }

    #[test]
    fn get_lib_files_to_skip_all_features() {
        let skip = get_lib_files_to_skip("typescript", true, true, true);
        assert!(skip.is_empty());
    }

    #[test]
    fn get_lib_files_to_skip_javascript() {
        let skip = get_lib_files_to_skip("javascript", true, true, true);
        assert!(skip.contains(&"tsconfig.json"));
    }
```

**Step 3: Run tests**

Run: `cargo test`
Expected: All tests pass (new + existing).

**Step 4: Commit**

```bash
git add src/features.rs
git commit -m "feat: add lib feature skip and package.json processing"
```

---

### Task 4: Wire up main.rs for project type branching

**Files:**
- Modify: `src/main.rs`

**Step 1: Update imports and cmd_init**

Replace the entire `src/main.rs` with the branching logic. Key changes:

1. Import new types: `ProjectType`, `Language`, `LibFeature`, `prompt_project_type`, `prompt_language`, `prompt_lib_features`, `outro_success_lib`
2. Import new features: `get_lib_files_to_skip`, `update_lib_package_json`
3. Import `default_template_url` from `cli`
4. After project name validation, determine project type (from flag or prompt)
5. Resolve template URL: `args.template.unwrap_or_else(|| default_template_url(is_lib).to_string())`
6. Branch into `cmd_init_app` and `cmd_init_lib` helper functions

Updated `main.rs`:

```rust
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
```

**Step 2: Compile and run tests**

Run: `cargo test`
Expected: All tests pass, compiles cleanly.

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire up project type branching in init flow"
```

---

### Task 5: Create the lib-template repository

**Files:**
- Create: new repo `nullslate/lib-template` with all template files

This task creates the actual template repo with all features included (the CLI strips what's not needed).

**Step 1: Create the repository locally**

Create a new directory and initialize it:

```bash
mkdir -p ~/Dev/projects/libs/lib-template
cd ~/Dev/projects/libs/lib-template
git init
```

**Step 2: Create package.json**

```json
{
  "name": "{{project_name}}",
  "version": "0.1.0",
  "private": false,
  "type": "module",
  "main": "./dist/index.js",
  "module": "./dist/index.mjs",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.js",
      "types": "./dist/index.d.ts"
    },
    "./styles": "./dist/styles/index.css"
  },
  "files": ["dist"],
  "scripts": {
    "dev": "vite",
    "build": "tsup",
    "test": "vitest run",
    "test:watch": "vitest",
    "lint": "tsc --noEmit"
  },
  "peerDependencies": {
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@tailwindcss/vite": "^4.1.18",
    "@testing-library/jest-dom": "^6.6.3",
    "@testing-library/react": "^16.3.0",
    "@types/react": "^19.1.2",
    "@types/react-dom": "^19.1.2",
    "jsdom": "^26.1.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "tailwindcss": "^4.1.18",
    "tsup": "^8.5.0",
    "typescript": "^5.8.3",
    "vite": "^7.1.7",
    "vitest": "^3.2.1"
  }
}
```

**Step 3: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "./dist",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "esModuleInterop": true,
    "verbatimModuleSyntax": true
  },
  "include": ["src"],
  "exclude": ["node_modules", "dist"]
}
```

**Step 4: Create tsup.config.ts**

```typescript
import { defineConfig } from "tsup"

export default defineConfig({
  entry: ["src/index.ts"],
  format: ["cjs", "esm"],
  dts: true,
  sourcemap: true,
  clean: true,
  external: ["react", "react-dom"],
})
```

**Step 5: Create vite.config.ts**

```typescript
import { defineConfig } from "vite"

export default defineConfig({
  build: {
    lib: {
      entry: "src/index.ts",
      formats: ["es", "cjs"],
    },
  },
})
```

**Step 6: Create vitest.config.ts**

```typescript
import { defineConfig } from "vitest/config"

export default defineConfig({
  test: {
    environment: "jsdom",
    globals: true,
  },
})
```

**Step 7: Create src/index.ts**

```typescript
export { cn } from "./utils"
```

**Step 8: Create src/utils.ts**

```typescript
export function cn(...classes: (string | undefined | null | false)[]): string {
  return classes.filter(Boolean).join(" ")
}
```

**Step 9: Create src/components/button.tsx** (React feature)

```tsx
import type { ButtonHTMLAttributes } from "react"

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary"
}

export function Button({ variant = "primary", className, ...props }: ButtonProps) {
  return (
    <button
      className={`btn btn-${variant} ${className ?? ""}`}
      {...props}
    />
  )
}
```

Update `src/index.ts` to also export components:

```typescript
export { cn } from "./utils"
export { Button } from "./components/button"
```

**Step 10: Create src/styles/index.css** (CSS feature)

```css
@import "tailwindcss";

.btn {
  @apply inline-flex items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors;
}

.btn-primary {
  @apply bg-blue-600 text-white hover:bg-blue-700;
}

.btn-secondary {
  @apply bg-gray-200 text-gray-900 hover:bg-gray-300;
}
```

**Step 11: Create src/__tests__/utils.test.ts** (Testing feature)

```typescript
import { describe, it, expect } from "vitest"
import { cn } from "../utils"

describe("cn", () => {
  it("joins class names", () => {
    expect(cn("a", "b")).toBe("a b")
  })

  it("filters falsy values", () => {
    expect(cn("a", false, null, undefined, "b")).toBe("a b")
  })

  it("returns empty string for no args", () => {
    expect(cn()).toBe("")
  })
})
```

**Step 12: Create .gitignore**

```
node_modules/
dist/
*.tsbuildinfo
.DS_Store
bun.lock
package-lock.json
```

**Step 13: Create .npmrc**

```
@thesandybridge:registry=https://npm.pkg.github.com
```

**Step 14: Create CLAUDE.md**

```markdown
# {{project_name}} — Claude Code Instructions

## Stack
- **Build:** tsup (production), Vite (development)
- **Language:** TypeScript
- **Output:** Dual ESM/CJS with type declarations

## Commands
- `bun run build` — production build with tsup
- `bun run dev` — Vite dev server
- `bun run test` — run tests with Vitest
- `bun run lint` — type check with tsc

## Key File Paths
```
src/index.ts        — main export barrel
src/utils.ts        — utility functions
src/components/     — React components (if React enabled)
src/styles/         — CSS/Tailwind styles (if CSS enabled)
src/__tests__/      — test files (if testing enabled)
tsup.config.ts      — build configuration
vite.config.ts      — dev server configuration
```

## Code Style
- Prefer editing existing files over creating new ones
- No unnecessary comments or docstrings

## Git
- Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `build:`, `perf:`
- No Co-Authored-By lines
```

**Step 15: Create GitHub repo, commit, and push**

```bash
cd ~/Dev/projects/libs/lib-template
git add .
git commit -m "feat: initial lib template with react, css, and testing features"
gh repo create nullslate/lib-template --public --source=. --push
```

---

### Task 6: Test the full flow end-to-end

**Step 1: Build and install the CLI**

```bash
cd ~/Dev/projects/libs/create-sandybridge-app
cargo install --path .
```

**Step 2: Test app scaffold (regression)**

```bash
cd /tmp
ns init test-app -y --no-install --no-git
```

Expected: Scaffolds app template as before, no errors.

```bash
rm -rf /tmp/test-app
```

**Step 3: Test lib scaffold with all features**

```bash
ns init test-lib -y --lib --react --css --testing --no-install --no-git
```

Expected: Scaffolds lib template with all features, includes `src/components/`, `src/styles/`, `vitest.config.ts`, `tsconfig.json`.

**Step 4: Test lib scaffold minimal (no features, TypeScript)**

```bash
ns init test-lib-min -y --lib --no-install --no-git
```

Expected: No `src/components/`, no `src/styles/`, no `vitest.config.ts`. Has `tsconfig.json` and `src/index.ts`.

**Step 5: Test lib scaffold JavaScript**

```bash
ns init test-lib-js -y --lib --lang javascript --no-install --no-git
```

Expected: No `tsconfig.json`.

**Step 6: Clean up test dirs**

```bash
rm -rf /tmp/test-app /tmp/test-lib /tmp/test-lib-min /tmp/test-lib-js
```

**Step 7: Commit any fixes discovered during testing**

---

### Task 7: Update integration tests

**Files:**
- Modify: `tests/integration.rs`

**Step 1: Add lib scaffold integration test**

Add to `tests/integration.rs`:

```rust
#[test]
#[ignore] // requires network for git clone
fn scaffold_lib_project() {
    let bin = cargo_bin();
    let dir = tempfile::tempdir().unwrap();
    let project_path = dir.path().join("test-lib");

    let status = Command::new(&bin)
        .args([
            "init",
            "test-lib",
            "--lib",
            "--no-git",
            "--no-install",
            "--react",
            "--testing",
            "-y",
            "--path",
            project_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run CLI");

    assert!(status.success(), "CLI exited with error");
    assert!(project_path.exists(), "Project directory not created");
    assert!(
        project_path.join("package.json").exists(),
        "package.json not found"
    );
    assert!(
        project_path.join("tsup.config.ts").exists(),
        "tsup.config.ts not found"
    );
    assert!(
        project_path.join("src/components").exists(),
        "React components should be included"
    );
    assert!(
        project_path.join("vitest.config.ts").exists(),
        "vitest.config.ts should be included"
    );
    assert!(
        !project_path.join("src/styles").exists(),
        "CSS should be excluded (not requested)"
    );
}

#[test]
#[ignore] // requires network for git clone
fn scaffold_minimal_lib() {
    let bin = cargo_bin();
    let dir = tempfile::tempdir().unwrap();
    let project_path = dir.path().join("test-lib-min");

    let status = Command::new(&bin)
        .args([
            "init",
            "test-lib-min",
            "--lib",
            "--no-git",
            "--no-install",
            "-y",
            "--path",
            project_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run CLI");

    assert!(status.success(), "CLI exited with error");
    assert!(
        !project_path.join("src/components").exists(),
        "React should be excluded"
    );
    assert!(
        !project_path.join("src/styles").exists(),
        "CSS should be excluded"
    );
    assert!(
        !project_path.join("vitest.config.ts").exists(),
        "Testing should be excluded"
    );
    assert!(
        project_path.join("tsconfig.json").exists(),
        "TypeScript should be default"
    );
}
```

**Step 2: Run tests (unit only, integration requires network)**

Run: `cargo test`
Expected: All unit tests pass.

**Step 3: Commit**

```bash
git add tests/integration.rs
git commit -m "test: add integration tests for lib scaffold"
```

---

### Task 8: Final commit and push

**Step 1: Push CLI changes**

```bash
cd ~/Dev/projects/libs/create-sandybridge-app
git push
```

**Step 2: Rebuild CLI**

```bash
cargo install --path .
```
