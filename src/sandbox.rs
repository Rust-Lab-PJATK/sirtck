use anyhow::{bail, Context, Result};
use std::{
    env,
    io::Read,
    path::Path,
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

pub struct Sandbox {
    image: String,
    timeout: Duration,
    cpus: u32,
    memory_mb: u32,
}

impl Sandbox {
    pub fn from_env() -> Result<Self> {
        fn positive(name: &str, default: u32) -> Result<u32> {
            let value = match env::var(name) {
                Ok(value) => value.parse().with_context(|| format!("Invalid {name}"))?,
                Err(env::VarError::NotPresent) => default,
                Err(error) => return Err(error.into()),
            };

            anyhow::ensure!(value > 0, "{name} must be positive");

            Ok(value)
        }

        Ok(Self {
            image: env::var("SIRTCK_SANDBOX_IMAGE").unwrap_or_else(|_| "sirtck-runner:1".into()),
            timeout: Duration::from_secs(positive("SIRTCK_TIMEOUT_SECONDS", 120)?.into()),
            cpus: positive("SIRTCK_CPUS", 2)?,
            memory_mb: positive("SIRTCK_MEMORY_MB", 1024)?,
        })
    }

    fn command(&self, root: &Path, task: &str, name: &str, args: &[&str]) -> Command {
        let mut command = Command::new("docker");
        command.args([
            "run",
            "--rm",
            "--pull=never",
            "--name",
            name,
            "--network=none",
            "--read-only",
            "--user=10001:10001",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--pids-limit=128",
            "--ulimit=nofile=1024:1024",
            "--log-driver=none",
            "--cpus",
            &self.cpus.to_string(),
            "--memory",
            &format!("{}m", self.memory_mb),
            "--memory-swap",
            &format!("{}m", self.memory_mb),
            "--tmpfs=/tmp:rw,nosuid,nodev,size=64m,mode=1777",
            "--tmpfs=/target:rw,exec,nosuid,nodev,size=512m,uid=10001,gid=10001",
            "--env=CARGO_TARGET_DIR=/target",
            "--env=CARGO_NET_OFFLINE=true",
            "--env=RUSTUP_TOOLCHAIN=1.99.1",
            "--mount",
        ]);
        command.arg(format!(
            "type=bind,src={},dst=/submission,readonly",
            root.display()
        ));
        command.args([
            "--workdir",
            &format!("/submission/{task}"),
            "--entrypoint=cargo",
            &self.image,
        ]);
        command.args(args);
        command
    }

    pub fn run(&self, root: &Path, task: &str, args: &[&str]) -> Result<Output> {
        let id = tempfile::tempdir()?;
        let name = format!(
            "sirtck-{}",
            id.path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .replace('.', "")
        );

        let mut command = self.command(root, task, &name, args);
        let output = run_bounded(&mut command, self.timeout);

        // Removing the container also kills descendants when the client times out.
        let cleanup = Command::new("docker")
            .args(["rm", "--force", &name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        let output = output.context("Sandbox execution failed (container cleanup attempted)")?;

        if matches!(output.status.code(), Some(125..=127) | Some(137)) {
            bail!(
                "Docker sandbox failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        cleanup.context("Failed to invoke Docker cleanup")?;

        Ok(output)
    }
}

fn run_bounded(command: &mut Command, timeout: Duration) -> Result<Output> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Cannot start process; verify Docker installation and daemon access")?;

    // Drain both streams concurrently; retain at most 32 KiB per stream.
    fn drain(mut stream: impl Read) -> Vec<u8> {
        let mut output = Vec::new();
        let mut buffer = [0; 4096];

        while let Ok(count) = stream.read(&mut buffer) {
            if count == 0 {
                break;
            }

            let keep = count.min(32768usize.saturating_sub(output.len()));

            output.extend_from_slice(&buffer[..keep]);
        }

        output
    }

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out = thread::spawn(move || drain(stdout));
    let err = thread::spawn(move || drain(stderr));

    let start = Instant::now();

    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Output {
                status,
                stdout: out.join().unwrap_or_default(),
                stderr: err.join().unwrap_or_default(),
            });
        }

        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            bail!("Sandbox exceeded {} seconds", timeout.as_secs());
        }

        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_enforces_isolation() {
        let sandbox = Sandbox {
            image: "runner:1".into(),
            timeout: Duration::from_secs(1),
            cpus: 2,
            memory_mb: 1024,
        };
        let cmd = sandbox.command(
            Path::new("/tmp/project"),
            "task",
            "test",
            &["test", "--offline", "--locked"],
        );
        let args: Vec<_> = cmd
            .get_args()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        for required in [
            "--network=none",
            "--read-only",
            "--user=10001:10001",
            "--cap-drop=ALL",
            "--pull=never",
            "--offline",
            "--locked",
        ] {
            assert!(args.iter().any(|arg| arg == required));
        }
        assert!(args
            .iter()
            .any(|arg| arg == "type=bind,src=/tmp/project,dst=/submission,readonly"));
    }

    #[test]
    fn timeout_terminates_process() {
        let error =
            run_bounded(Command::new("sleep").arg("5"), Duration::from_millis(30)).unwrap_err();
        assert!(error.to_string().contains("exceeded"));
    }

    #[test]
    fn missing_executable_is_an_error() {
        assert!(run_bounded(
            &mut Command::new("/nonexistent/sirtck"),
            Duration::from_secs(1)
        )
        .is_err());
    }
}
