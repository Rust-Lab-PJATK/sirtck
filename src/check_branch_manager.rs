use octocrab::models::pulls::PullRequest;
use crate::entity::score::Score;
use crate::entity::task::Task;
use std::process::Command;
use std::fs;
use tempfile::tempdir;
use fs_extra::dir::{copy as copy_dir, CopyOptions};
use regex::Regex;

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
        let passed: u32 = caps.get(1)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let failed: u32 = caps.get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);
        let measured: u32 = caps.get(4)
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
            repo.html_url.as_ref().expect("Expected html_url to be present").to_string()
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
/// In case of errors (e.g. failing to clone the repository or copy test files),
/// an empty/default score is returned.
pub fn check_branch_via_task(task: &Task, pr: &PullRequest) -> Score {
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path();

    let repo_url = match get_repo_url(pr) {
        Some(url) => url,
        None => {
            eprintln!("Repository URL not found");
            return Score::default();
        }
    };

    let branch = &pr.head.ref_field;
    let clone_status = Command::new("git")
        .args(&["clone", "--branch", branch, &repo_url, temp_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute git clone");
    if !clone_status.success() {
        eprintln!("Repository cloning failed");
        return Score::default();
    }

    let tests_source = std::path::Path::new(&task.test_path);
    let destination = temp_path.join(&task.id);
    fs::create_dir_all(&destination)
        .expect("Failed to create destination directory");

    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;
    if let Err(err) = copy_dir(&tests_source, &destination, &copy_options) {
        eprintln!("Error copying test files: {:?}", err);
        return Score::default();
    }

    let test_output = Command::new("cargo")
        .args(&["test", "--", "--nocapture"])
        .current_dir(&destination)
        .output()
        .expect("Failed to execute cargo test");

    let test_stdout = String::from_utf8_lossy(&test_output.stdout);
    let (total_tests, passed_tests) = parse_test_summary(&test_stdout);

    let clippy_output = Command::new("cargo")
        .args(&["clippy", "--", "-D", "warnings"])
        .current_dir(&destination)
        .output()
        .expect("Failed to execute cargo clippy");
    let clippy_stderr = String::from_utf8_lossy(&clippy_output.stderr);

    Score {
        max_points: total_tests,
        received_points: passed_tests,
        test_output: test_stdout.to_string(),
        clippy_output: clippy_stderr.to_string(),
    }
}
