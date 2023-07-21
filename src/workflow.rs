use std::collections::HashMap;

use itertools::Itertools;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")] // can not deny unknown fields to support YAML anchors
/// The entire workflow definition.
pub(crate) struct Workflow {
    /// The version of this workflow file (major.minor).
    pub version: String,
    /// RegEx for capturing env vars at plan time + baking these into the plan.
    pub capture: Option<String>,
    /// Explicitly set env vars.
    pub env: Option<HashMap<String, String>>,
    /// All nodes.
    pub nodes: HashMap<String, Node>,
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// A task execution environment.
pub(crate) struct Shell {
    /// The program (like "/bin/bash").
    pub program: String,
    /// Custom args (like \["-c"\]).
    pub args: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// An individual node for executing a task batch.
pub(crate) struct Node {
    /// A description of this node.
    pub description: Option<String>,
    /// Reference nodes that need to be executed prior to this one.
    pub pre: Option<Vec<String>>,

    /// An n-dimensional matrix that is executed for every item in its cartesian product.
    pub matrix: Option<Matrix>,
    /// The tasks to be executed.
    pub tasks: Vec<Task>,

    /// RegEx for capturing env vars at plan time + baking these into the plan.
    pub capture: Option<String>,
    /// Explicitly set env vars.
    pub env: Option<HashMap<String, String>>,
    /// Custom program to execute the scripts.
    pub shell: Option<Shell>,
    /// Custom workdir.
    pub workdir: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// An entry in the n-dimensional matrix for the node execution.
pub(crate) enum Matrix {
    Dense {
        drop: Option<String>,
        dimensions: Vec<Vec<MatrixCell>>,
    },
    Sparse {
        dimensions: Vec<Vec<MatrixCell>>,
        keep: Option<String>,
    },
}

impl Matrix {
    pub(crate) fn compile(&self) -> Result<Vec<crate::plan::Invocation>, crate::error::Error> {
        let (dimensions, regex, invert) = match self {
            | Self::Dense { drop, dimensions } => (dimensions, drop, true),
            | Self::Sparse { keep, dimensions } => (dimensions, keep, false),
        };

        let regex = fancy_regex::Regex::new(match regex {
            | Some(s) => s,
            | None => ".*",
        })?;

        // Bake the coords in their respective dimension into the struct itself.
        // This makes coord finding for regex (later) a breeze.
        let dims_widx = dimensions.iter().map(|d_x| {
            let mut y = 0usize;
            d_x.iter()
                .map(|d_y| {
                    y += 1;
                    (y - 1, d_y)
                })
                .collect_vec()
        });

        let cp = dims_widx.multi_cartesian_product();
        let mut v = Vec::<crate::plan::Invocation>::new();

        for next in cp {
            let coords = next.iter().map(|v| format!("{}", v.0)).join(",");

            // Use the baked-in coords to invoke regex.
            if regex.is_match(&format!("{}", coords))? ^ !invert {
                continue;
            }

            let mut env = HashMap::<String, String>::new();
            for m in next {
                if let Some(e) = &m.1.env {
                    env.extend(e.clone());
                }
            }

            v.push(crate::plan::Invocation { env, coords });
        }
        Ok(v)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// An entry in the n-dimensional matrix for the node execution.
pub(crate) struct MatrixCell {
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
/// An individual task.
pub(crate) struct Task {
    /// The script content to execute. Can contain handlebars placeholders.
    pub script: String,

    /// Explicitly set env vars.
    pub env: Option<HashMap<String, String>>,
    /// Custom program to execute the scripts.
    pub shell: Option<Shell>,
    /// Custom workdir.
    pub workdir: Option<String>,
}
