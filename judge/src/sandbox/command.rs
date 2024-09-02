use std::{fmt, path::PathBuf, process::ExitStatus, str};

use serde::Deserialize;
use tokio::process::Command as TokioCommand;
use which::which;

use super::resource::ResourceUsage;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<String>")]
pub struct Command {
    executable: PathBuf,
    args: Vec<String>,
}

impl Command {
    pub fn create(&self) -> TokioCommand {
        let mut command = TokioCommand::new(&self.executable);
        command.args(&self.args);
        command
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.executable.display(), self.args.join(" "))
    }
}

#[derive(Debug)]
pub struct EmptyCommand;

impl fmt::Display for EmptyCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("empty command", f)
    }
}

impl TryFrom<Vec<String>> for Command {
    type Error = EmptyCommand;

    fn try_from(vec: Vec<String>) -> Result<Self, Self::Error> {
        let (executable, args) = vec.split_first().ok_or(EmptyCommand)?;
        Ok(Command {
            executable: which(executable).unwrap_or_else(|_| PathBuf::from(executable)),
            args: args.to_owned(),
        })
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
        str::from_utf8(self.stdout())
    }

    pub fn stderr(&self) -> &[u8] {
        &self.stderr
    }

    pub fn stderr_utf8(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.stderr())
    }

    pub fn resource_usage(&self) -> ResourceUsage {
        self.resource_usage
    }
}
