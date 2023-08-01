include!("check_features.rs");

pub mod args;
pub mod compiler;
pub mod error;
pub mod exec;
pub mod plan;
pub mod reference;
pub mod workflow;

use crate::{compiler::Compiler, workflow::Workflow};
use anyhow::Result;
use args::{InitOutput, ManualFormat};
use exec::{ExecutionEngine, OutputMode};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = crate::args::ClapArgumentLoader::load()?;

    match cmd.command {
        | crate::args::Command::Manual { path, format } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            match format {
                | ManualFormat::Manpages => {
                    reference::build_manpages(&out_path)?;
                },
                | ManualFormat::Markdown => {
                    reference::build_markdown(&out_path)?;
                },
            }
            Ok(())
        },
        | crate::args::Command::Autocomplete { path, shell } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            reference::build_shell_completion(&out_path, &shell)?;
            Ok(())
        },
        | crate::args::Command::WorkflowInit { template, output } => {
            match output {
                | InitOutput::File(f) => std::fs::write(f, template.render())?,
                | InitOutput::Stdout => print!("{}", template.render()),
            };
            Ok(())
        },
        | crate::args::Command::WorkflowSchema => {
            print!(
                "{}",
                serde_json::to_string_pretty(&schemars::schema_for!(crate::workflow::Workflow)).unwrap()
            );
            Ok(())
        },
        | crate::args::Command::Execute {
            plan,
            workers,
            no_stdout,
            no_stderr,
        } => {
            let exec_engine = ExecutionEngine::new(OutputMode {
                stdout: !no_stdout,
                stderr: !no_stderr,
            });
            exec_engine.execute(plan, workers).await?;
            Ok(())
        },
        | crate::args::Command::Plan {
            workflow,
            nodes,
            args,
            format,
        } => {
            let w = Workflow::load(&workflow)?;
            let nodes = nodes.compile(&w)?;
            let c = Compiler::new(w);
            let x = c.plan(&nodes, &args).await?;
            print!("{}", format.serialize(&x)?);
            Ok(())
        },
        | crate::args::Command::List { workflow, format } => {
            let w = Workflow::load(&workflow)?;
            let c = Compiler::new(w);
            c.list(&format).await?;
            Ok(())
        },
        | crate::args::Command::Describe {
            workflow,
            nodes,
            format,
        } => {
            let w = Workflow::load(&workflow)?;
            let nodes = nodes.compile(&w)?;
            let c = Compiler::new(w);
            c.describe(&nodes, &format).await?;
            Ok(())
        },
    }
}
