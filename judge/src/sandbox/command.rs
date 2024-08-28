use std::{
    ffi::{OsStr, OsString},
    fmt,
    path::{Path, PathBuf},
    process::ExitStatus,
    str,
};

use tokio::process::Command as TokioCommand;

use super::resource::ResourceUsage;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Command {
    executable: PathBuf,
    args: Vec<OsString>,
}

impl Command {
    pub fn new(
        executable: impl AsRef<Path>,
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> Self {
        Command {
            executable: executable.as_ref().to_path_buf(),
            args: args.into_iter().map(|s| s.as_ref().to_owned()).collect(),
        }
    }

    pub fn create(&self) -> TokioCommand {
        let mut command = TokioCommand::new(&self.executable);
        command.args(&self.args);
        command
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}",
            self.executable.display(),
            Vec::from_iter(self.args.iter().map(|s| s.to_string_lossy())).join(", ")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    exit_status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    resource_usage: ResourceUsage,
}

impl Output {
    pub fn new(
        exit_status: ExitStatus,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        resource_usage: ResourceUsage,
    ) -> Self {
        Self {
            exit_status,
            stdout,
            stderr,
            resource_usage,
        }
    }

    pub fn exit_status(&self) -> ExitStatus {
        self.exit_status
    }

    pub fn stdout(&self) -> &[u8] {
        &self.stdout
    }

    pub fn stdout_utf8(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.stdout)
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn stderr_utf8(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(&self.stderr)
    }

    pub fn resource_usage(&self) -> ResourceUsage {
        self.resource_usage
    }
}
