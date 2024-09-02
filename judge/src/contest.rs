use serde::Deserialize;

use crate::sandbox::{Command, ResourceLimits};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Contest {
    pub name: String,
    pub tasks: Vec<Task>,
    #[serde(rename = "judge")]
    pub config: Config,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Task {
    pub subtasks: Vec<Subtask>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Subtask {
    pub tests: Vec<Test>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Test {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub skip_count: u8,
    pub resource_limits: ResourceLimits,
    #[serde(alias = "language")]
    pub languages: Vec<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Language {
    pub name: String,
    pub filename: String,
    pub compile: Option<Command>,
    pub run: Command,
}

impl Contest {
    pub fn load(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
