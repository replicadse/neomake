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
    pub run: Vec<String>,
}

pub(crate) fn default_config() -> String {
    r###"version: 1.0

env:
  DEFAULT_ENV_VAR: default var
  OVERRIDE_ENV_VAR_0: old e0
  OVERRIDE_ENV_VAR_1: old e1

.cargo-build: &cargo-build
  - cargo build $CARGO_ARGS

chains:
  test:
    matrix:
      - env:
          OVERRIDE_ENV_VAR_0: new e0
    tasks:
      - env:
          OVERRIDE_ENV_VAR_1: new e1
        run:
          - printf "$DEFAULT_ENV_VAR"
          - printf "$OVERRIDE_ENV_VAR_0"
          - printf "$OVERRIDE_ENV_VAR_1"
          - printf "{{ args.test }}" # this will require an argument to be passed via '-a args.test="some-argument"'
          - unknown-command
          - printf "too far!"
"###
    .to_owned()
}
