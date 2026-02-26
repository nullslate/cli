use std::process::Command;

fn cargo_bin() -> String {
    let output = Command::new("cargo")
        .args(["build", "--message-format=short"])
        .output()
        .expect("Failed to build");
    assert!(output.status.success(), "cargo build failed");

    format!(
        "{}/target/debug/nullslate",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
#[ignore] // requires network for git clone
fn scaffold_minimal_project() {
    let bin = cargo_bin();
    let dir = tempfile::tempdir().unwrap();
    let project_path = dir.path().join("test-minimal");

    let status = Command::new(&bin)
        .args([
            "init",
            "test-minimal",
            "--no-git",
            "--no-install",
            "--no-auth",
            "--db",
            "none",
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
        project_path.join("vite.config.ts").exists()
            || project_path.join("index.html").exists(),
        "Expected Vite project files"
    );
    assert!(
        !project_path.join("src/lib/auth.ts").exists(),
        "auth.ts should be skipped"
    );
    assert!(
        !project_path.join(".env").exists(),
        ".env should not exist without auth"
    );
}

#[test]
#[ignore] // requires network for git clone
fn scaffold_with_auth() {
    let bin = cargo_bin();
    let dir = tempfile::tempdir().unwrap();
    let project_path = dir.path().join("test-auth");

    let status = Command::new(&bin)
        .args([
            "init",
            "test-auth",
            "--no-git",
            "--no-install",
            "--db",
            "none",
            "-y",
            "--path",
            project_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run CLI");

    assert!(status.success(), "CLI exited with error");
    assert!(project_path.join(".env").exists(), ".env should exist with auth");

    let env_content = std::fs::read_to_string(project_path.join(".env")).unwrap();
    assert!(
        env_content.contains("AUTH_SECRET="),
        ".env should contain AUTH_SECRET"
    );
    assert!(
        env_content.contains("AUTH_GITHUB_ID="),
        ".env should contain AUTH_GITHUB_ID"
    );
    assert!(
        !env_content.contains("DATABASE_URL"),
        ".env should not contain DATABASE_URL"
    );
    assert!(
        !env_content.contains("JWT_SECRET"),
        ".env should not contain JWT_SECRET"
    );
}

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
