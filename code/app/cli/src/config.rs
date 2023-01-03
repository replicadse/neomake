use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub version: String,
    pub chains: HashMap<String, Chain>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Chain {
    pub matrix: Vec<MatrixEntry>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MatrixEntry {
    pub workdir: Option<String>,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Task {
    pub run: Vec<String>,
}

pub(crate) fn default_config() -> String {
    r###"version: 1.0.0

.cargo-build: &cargo-build
  - cargo build $CARGO_ARGS

chains:
  "app/cli":
    matrix:
      - workdir: ./code/apps/cli
        env:
          CARGO_ARGS: ""
      - workdir: ./code/apps/cli
        env:
          CARGO_ARGS: "--release"
    tasks:
      - run: *cargo-build
"###
    .to_owned()
}
