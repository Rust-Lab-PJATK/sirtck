use crate::entity::score::Score;
use crate::entity::task::Task;
use crate::sandbox::Sandbox;
use anyhow::{ensure, Context, Result};
use fs_extra::dir::{copy as copy_dir, CopyOptions};
use octocrab::models::pulls::PullRequest;
use regex::Regex;
use std::process::Command;
use tempfile::tempdir;

fn reject_symlinks(path: &std::path::Path) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "Symlinks in submissions are not supported"
        );
        if kind.is_dir() {
            reject_symlinks(&entry.path())?;
        }
    }
    Ok(())
}

/// Parses the test summary output and returns a tuple of (total_tests, passed_tests).
///
/// The expected format is:
/// "test result: ... X passed; Y failed; Z ignored; A measured; B filtered out"
pub fn parse_test_summary(output: &str) -> (u32, u32) {
    let pattern = r"test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out";
    let re = Regex::new(pattern).expect("Invalid regex pattern");

    let mut total_tests = 0;
    let mut total_passed = 0;

    for caps in re.captures_iter(output) {
        let passed: u32 = caps
            .get(1)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let failed: u32 = caps
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let measured: u32 = caps
            .get(4)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let tests_run = passed + failed + measured;
        if tests_run == 0 {
            continue;
        }
        total_tests += tests_run;
        total_passed += passed;
    }

    (total_tests, total_passed)
}

/// Retrieves the repository URL from a PullRequest.
/// Prefers the clone URL, then the SSH URL, and falls back to the HTML URL.
fn get_repo_url(pr: &PullRequest) -> Option<String> {
    pr.base.repo.as_ref().map(|repo| {
        if let Some(clone_url) = &repo.clone_url {
            clone_url.to_string()
        } else if let Some(ssh_url) = &repo.ssh_url {
            ssh_url.to_string()
        } else {
            repo.html_url
                .as_ref()
                .expect("Expected html_url to be present")
                .to_string()
        }
    })
}

/// Checks out the branch indicated by the pull request and runs tests and clippy checks based on the provided task.
///
/// It returns a `Score` containing:
/// - `max_points`: the total number of tests run,
/// - `received_points`: the number of passed tests,
/// - `test_output`: the raw output from running the tests,
/// - `clippy_output`: the Clippy stderr output.
///
/// Infrastructure failures are returned separately from submission results.
pub fn check_branch_via_task(task: &Task, pr: &PullRequest) -> Result<Score> {
    let sandbox = Sandbox::from_env()?;
    ensure!(
        !task.id.is_empty()
            && task
                .id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_'),
        "Invalid task identifier"
    );
    let temp_dir = tempdir()?;
    let temp_path = temp_dir.path();

    let repo_url = match get_repo_url(pr) {
        Some(url) => url,
        None => {
            eprintln!("Repository URL not found");
            anyhow::bail!("Repository URL not found");
        }
    };

    let branch = &pr.head.ref_field;
    let clone_status = Command::new("git")
        .args([
            "clone",
            "--branch",
            branch,
            &repo_url,
            temp_path.to_str().unwrap(),
        ])
        .status()
        .context("Failed to execute git clone")?;
    if !clone_status.success() {
        eprintln!("Repository cloning failed");
        anyhow::bail!("Repository cloning failed");
    }

    let tests_source = std::path::Path::new(&task.test_path);
    reject_symlinks(temp_path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(temp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    let destination = temp_path.join(&task.id);

    if !destination.is_dir() {
        eprintln!("Exercise directory not found: {}", destination.display());
        anyhow::bail!("Exercise directory not found");
    }

    if !tests_source.is_dir() {
        eprintln!("Test directory not found: {}", tests_source.display());
        anyhow::bail!("Test directory not found");
    }

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;

    if let Err(err) = copy_dir(tests_source, &destination, &copy_options) {
        eprintln!("Error copying test files: {:?}", err);
        anyhow::bail!("Error copying test files: {err}");
    }

    let test_output = sandbox.run(
        temp_path,
        &task.id,
        &["test", "--offline", "--locked", "--", "--nocapture"],
    )?;

    let test_stdout = String::from_utf8_lossy(&test_output.stdout);
    let (total_tests, passed_tests) = parse_test_summary(&test_stdout);
    ensure!(
        test_output.status.success() || total_tests > 0,
        "Cargo test could not complete (check Cargo.lock and offline dependencies): {}",
        String::from_utf8_lossy(&test_output.stderr)
    );

    let clippy_output = sandbox.run(
        temp_path,
        &task.id,
        &["clippy", "--offline", "--locked", "--", "-D", "warnings"],
    )?;

    let clippy_stderr = String::from_utf8_lossy(&clippy_output.stderr);

    Ok(Score {
        max_points: total_tests,
        received_points: passed_tests,
        test_output: test_stdout.to_string(),
        clippy_output: clippy_stderr.to_string(),
    })
}
