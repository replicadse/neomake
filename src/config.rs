use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")] // can not deny unknown fields to support YAML anchors
pub(crate) struct Config {
    pub version: String,
    pub env: Option<HashMap<String, String>>,
    pub chains: HashMap<String, Chain>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) struct Chain {
    pub description: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub workdir: Option<String>,
    pub pre: Option<Vec<String>>,
    pub matrix: Option<Vec<Vec<MatrixEntry>>>,
    pub tasks: Vec<Task>,
    pub shell: Option<Shell>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) struct Shell {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) struct MatrixEntry {
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub(crate) struct Task {
    pub workdir: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub shell: Option<Shell>,
    pub script: String,
}
