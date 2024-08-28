use serde::Deserialize;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Contest {
    pub name: String,
    pub tasks: Vec<Task>,
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

impl Contest {
    pub fn load(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
