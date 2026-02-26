use anyhow::Result;
use rand::Rng;
use std::fs;
use std::path::Path;

pub fn get_files_to_skip(include_docs: bool, include_auth: bool, include_db: bool) -> Vec<&'static str> {
    let mut skip = Vec::new();

    if !include_docs {
        skip.push("src/routes/docs");
        skip.push("content/docs");
        skip.push("src/lib/docs.ts");
        skip.push("src/lib/docs.test.ts");
        skip.push("src/lib/mdx-components.tsx");
        skip.push("src/components/docs-sidebar.tsx");
        skip.push("src/components/copyable-pre.tsx");
    }

    if !include_auth {
        skip.push("src/lib/auth.ts");
        skip.push("api/auth");
        skip.push("src/components/session-provider.tsx");
    }

    if !include_db {
        skip.push("src/lib/db.ts");
    }

    skip
}

pub fn should_skip_file(path: &str, skip_patterns: &[&str]) -> bool {
    for pattern in skip_patterns {
        if path.starts_with(pattern) || path == *pattern {
            return true;
        }
    }
    false
}

pub fn update_package_json(
    output_path: &Path,
    include_docs: bool,
    include_auth: bool,
    include_db: bool,
) -> Result<()> {
    let package_json_path = output_path.join("package.json");
    let content = fs::read_to_string(&package_json_path)?;
    let mut package: serde_json::Value = serde_json::from_str(&content)?;

    // Remove deps when features are OFF (template ships with all deps)
    if !include_docs {
        if let Some(deps) = package["dependencies"].as_object_mut() {
            deps.remove("@mdx-js/react");
            deps.remove("gray-matter");
        }
        if let Some(dev_deps) = package["devDependencies"].as_object_mut() {
            dev_deps.remove("@mdx-js/rollup");
            dev_deps.remove("remark-frontmatter");
            dev_deps.remove("remark-mdx-frontmatter");
        }
    }

    if !include_auth {
        if let Some(deps) = package["dependencies"].as_object_mut() {
            deps.remove("@auth/core");
        }
    }

    if !include_db {
        if let Some(deps) = package["dependencies"].as_object_mut() {
            deps.remove("pg");
        }
        if let Some(dev_deps) = package["devDependencies"].as_object_mut() {
            dev_deps.remove("@types/pg");
        }
    }

    let formatted = serde_json::to_string_pretty(&package)?;
    fs::write(&package_json_path, formatted)?;

    Ok(())
}

pub fn cleanup_layout_for_no_auth(output_path: &Path) -> Result<()> {
    let layout_path = output_path.join("src/routes/__root.tsx");
    if !layout_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&layout_path)?;

    // Remove SessionProvider import (no semicolons in new template)
    let content = content.replace(
        "import { SessionProvider } from \"@/components/session-provider\"\n",
        "",
    );

    // Remove SessionProvider wrapper (preserving inner content)
    let content = content.replace("        <SessionProvider>\n", "");
    let content = content.replace("        </SessionProvider>\n", "");

    fs::write(&layout_path, content)?;
    Ok(())
}

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
        if let Some(scripts) = package["scripts"].as_object_mut() {
            scripts.remove("test");
        }
    }

    let formatted = serde_json::to_string_pretty(&package)?;
    fs::write(&package_json_path, formatted)?;

    Ok(())
}

pub fn generate_random_secret() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn generate_env_file(output_path: &Path) -> Result<()> {
    let env_path = output_path.join(".env");
    let auth_secret = generate_random_secret();

    let env_content = format!(
        "# Auth (GitHub OAuth)\nAUTH_SECRET={}\nAUTH_GITHUB_ID=\nAUTH_GITHUB_SECRET=\n",
        auth_secret
    );

    fs::write(&env_path, env_content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_files_to_skip_no_features() {
        let skip = get_files_to_skip(false, false, false);
        assert!(skip.contains(&"src/routes/docs"));
        assert!(skip.contains(&"src/lib/auth.ts"));
        assert!(skip.contains(&"src/lib/db.ts"));
    }

    #[test]
    fn get_files_to_skip_all_features() {
        let skip = get_files_to_skip(true, true, true);
        assert!(skip.is_empty());
    }

    #[test]
    fn should_skip_file_exact_match() {
        assert!(should_skip_file("src/lib/db.ts", &["src/lib/db.ts"]));
    }

    #[test]
    fn should_skip_file_prefix_match() {
        assert!(should_skip_file("src/routes/docs/index.tsx", &["src/routes/docs"]));
    }

    #[test]
    fn should_skip_file_no_match() {
        assert!(!should_skip_file("src/main.tsx", &["src/lib/db.ts", "src/routes/docs"]));
    }

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

    #[test]
    fn generate_random_secret_length() {
        let secret = generate_random_secret();
        assert_eq!(secret.len(), 64); // 32 bytes * 2 hex chars
    }

    #[test]
    fn generate_random_secret_uniqueness() {
        let a = generate_random_secret();
        let b = generate_random_secret();
        assert_ne!(a, b);
    }
}
