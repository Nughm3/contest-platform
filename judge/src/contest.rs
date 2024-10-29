use schemars::JsonSchema;
use serde::Deserialize;

use crate::sandbox::ResourceLimits;

// NOTE: not all fields are used by the judge server, but are included to generate a JSON Schema

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Contest {
    pub name: String,
    pub duration: u32,
    pub submission_cooldown: u32,
    pub page: String,
    pub tasks: Vec<Task>,
    pub scoring: Scoring,
    #[serde(rename = "judge")]
    pub config: Config,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct Task {
    pub name: String,
    pub difficulty: Difficulty,
    pub answer: Option<String>,
    pub page: String,
    pub subtasks: Vec<Subtask>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, JsonSchema)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
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
pub struct Scoring {
    pub answer_score: u32,
    pub test_score: u32,
    pub subtask_score: u32,
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
