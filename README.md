# SIRTCK: Simple Interactive Rust Task Checker

SIRTCK is an automated system for evaluating Rust solutions via GitHub pull requests. It clones repositories, runs tests, lints the code with Clippy, and posts detailed feedback directly on the pull request—all with a fun, Rust-themed twist!

## Features

- **Automated Testing:** Executes Rust’s test suites to validate your code.
- **Linting with Clippy:** Ensures code quality and adherence to best practices.
- **Interactive Feedback:** Posts formatted comments (with Rust-themed visuals) on pull requests.
- **Task Management:** Reads task definitions from a JSON file and processes related PRs automatically.

## Installation

1. **Clone the repository:**

   ```bash
   git clone https://github.com/yourusername/sirtck.git
   cd sirtck
   ```

2. **Build the project:**

   ```bash
   cargo build --release
   ```

## Setup

1. **Create a `.env` File:**

   In the project root directory, create a file named `.env` and add your GitHub token:

   ```env
   GITHUB_TOKEN=ghp_yourGitHubPersonalAccessToken
   GITHUB_OWNER=owner
   GITHUB_REPO_NAME=test
   ```

2. **Prepare Task Definitions:**

   Ensure that your task definitions are available in a JSON file at `tasks/tasks.json`. Adjust the file path in the code if your file is located elsewhere.

## Usage

### Docker sandbox

Build the trusted runner before starting SIRTCK:

```bash
docker build -t sirtck-runner:1 .
```

SIRTCK requires a local Docker daemon accessible to its user. Each Cargo test
and Clippy invocation runs in a separate disposable non-root container with no
network, a read-only submission and filesystem, dropped capabilities, and limits
of 120 seconds, 2 CPUs, 1024 MiB RAM and 128 processes. Build artifacts use a
512 MiB tmpfs. No GitHub credentials or Docker socket are passed to submissions.

Optional configuration: `SIRTCK_SANDBOX_IMAGE` (default `sirtck-runner:1`),
`SIRTCK_TIMEOUT_SECONDS`, `SIRTCK_CPUS`, and `SIRTCK_MEMORY_MB` (positive integers).
Images are never pulled automatically. Cargo runs with `--offline --locked`;
submissions must include an up-to-date `Cargo.lock`. The base runner supports
dependency-free exercises. For other exercises, prepare a derived image with
`cargo fetch --locked` using reviewed manifests and lockfiles, then select it
with `SIRTCK_SANDBOX_IMAGE`. Missing cached dependencies produce a checking error.
Never prepare the trusted image by executing code or Cargo configuration from PRs.

Docker containers share the host kernel; use a dedicated worker host for hostile
submissions. Docker daemon access itself grants broad host privileges. Keep the
host and runner patched. Sandbox failures are posted without a numeric score
and are retried on the next checker run.

Run the application using Cargo:

```bash
cargo run --release
```

Upon execution, the system will:

- Load tasks from `tasks/tasks.json`.
- Use the GitHub API to locate pull requests associated with each task.
- Clone the relevant repository branches.
- Execute tests and perform Clippy checks.
- Post a formatted comment on each pull request with test results and Clippy output, including visual Rust-themed accents (e.g., 🦀).

An example comment posted on a PR might look like:

```markdown
## 🦀 Test Results 🦀

**Score:** 5/5

### Test Output
 (test output here)


### Clippy Output
   (clippy output here)
```
## Contributing

Contributions are welcome! Please open issues or submit pull requests to help improve SIRTCK.

## License

This project is licensed under the [MIT License](LICENSE).
