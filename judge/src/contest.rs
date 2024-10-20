use schemars::JsonSchema;
use serde::Deserialize;

use crate::sandbox::ResourceLimits;

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
pub struct Contest {
    pub name: String,
    pub tasks: Vec<Task>,
    #[serde(rename = "judge")]
    pub config: Config,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct Task {
    pub subtasks: Vec<Subtask>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct Subtask {
    pub tests: Vec<Test>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct Test {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub skip_count: u8,
    pub resource_limits: ResourceLimits,
    #[serde(alias = "language")]
    pub languages: Vec<Language>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct Language {
    pub name: String,
    pub filename: String,
    pub compile: Option<Vec<String>>,
    pub run: Vec<String>,
}

impl Contest {
    pub fn load(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
