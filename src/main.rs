include!("check_features.rs");

use std::path::PathBuf;
use std::result::Result;

use args::{InitOutput, ManualFormat};
use model::ExecEngine;

pub mod args;
pub mod error;
pub mod model;
pub mod output;
pub mod plan;
pub mod reference;
pub mod workflow;

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    let cmd = crate::args::ClapArgumentLoader::load()?;
    cmd.validate()?;

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
            prefix,
            silent,
        } => {
            let exec_engine = ExecEngine::new(prefix, silent);
            exec_engine.execute(plan, workers).await?;
            Ok(())
        },
        | crate::args::Command::Plan {
            workflow,
            nodes,
            args,
            format,
        } => {
            let m = model::Workflow::load(&workflow)?;
            let x = m.render_exec(&nodes, &args).await?;
            print!("{}", format.serialize(&x)?);
            Ok(())
        },
        | crate::args::Command::List { workflow, format } => {
            let m = model::Workflow::load(&workflow)?;
            m.list(&format).await?;
            Ok(())
        },
        | crate::args::Command::Describe {
            workflow,
            nodes,
            format,
        } => {
            let m = model::Workflow::load(&workflow)?;
            m.describe(&nodes, &format).await?;
            Ok(())
        },
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use interactive_process::InteractiveProcess;

    // const WORKFLOW_PATH: &'static str = "./test/.neomake.yaml";

    fn exec(command: &str) -> Result<String, crate::error::Error> {
        let mut cmd_proc = std::process::Command::new("sh");
        cmd_proc.arg("-c");
        cmd_proc.arg(command);

        let output = Arc::new(Mutex::new(Vec::<String>::new()));
        let output_fn = output.clone();

        let exit_status = InteractiveProcess::new(cmd_proc, move |l| match l {
            | Ok(v) => {
                output_fn.lock().unwrap().push(v);
            },
            | Err(e) => {
                output_fn.lock().unwrap().push(e.to_string());
            },
        })?
        .wait()?;

        let output_joined = output.lock().unwrap().join("\n");
        match exit_status.code().unwrap() {
            | 0 => Ok(output_joined),
            | _ => Err(crate::error::Error::Generic(output_joined)),
        }
    }

    #[test]
    fn test_workflow_init_min() {
        assert!(
            include_str!("../res/templates/min.yaml")
                == format!("{}\n", exec("cargo run -- workflow init -tmin -o-").unwrap())
        )
    }

    #[test]
    fn test_workflow_init_max() {
        assert!(
            include_str!("../res/templates/max.yaml")
                == format!("{}\n", exec("cargo run -- workflow init -tmax -o-").unwrap())
        )
    }

    #[test]
    fn test_smoke_workflow_schema() {
        exec("cargo run -- workflow init -tmax -o-").unwrap();
    }
}
