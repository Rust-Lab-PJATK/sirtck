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
