use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub version: String,
    pub env: Option<HashMap<String, String>>,
    pub chains: HashMap<String, Chain>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Chain {
    pub env: Option<HashMap<String, String>>,
    pub workdir: Option<String>,
    pub pre: Option<Vec<String>>,
    pub matrix: Option<Vec<MatrixEntry>>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MatrixEntry {
    pub workdir: Option<String>,
    pub env: Option<HashMap<String, String>>,
}
impl Default for MatrixEntry {
    fn default() -> Self {
        Self {
            env: None,
            workdir: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Task {
    pub workdir: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub script: String,
}

pub(crate) fn default_config() -> String {
    r###"version: 0.2

chains:
  minimal:
    tasks:
      - script: |
            set -e
            printf "first line"
            printf "second line"
"###
    .to_owned()
}
