mod args;
mod config;
mod error;

use std::{
    collections::{
        HashMap,
        VecDeque,
    },
    error::Error,
    result::Result,
};

use interactive_process::InteractiveProcess;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = args::ClapArgumentLoader::load()?;
    match args.command {
        | args::Command::Init => {
            init().await?;
            Ok(())
        },
        | args::Command::Run { config, chain, args } => {
            run(config, chain, args).await?;
            Ok(())
        },
    }
}

async fn init() -> Result<(), Box<dyn Error>> {
    std::fs::write("./.neomake.yaml", crate::config::default_config())?;
    Ok(())
}

async fn run(
    config: crate::config::Config,
    chain: String,
    args: HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    fn recursive_add(namespace: &mut std::collections::VecDeque<String>, parent: &mut serde_json::Value, value: &str) {
        let current_namespace = namespace.pop_front().unwrap();
        match namespace.len() {
            | 0 => {
                parent
                    .as_object_mut()
                    .unwrap()
                    .entry(&current_namespace)
                    .or_insert(serde_json::Value::String(value.to_owned()));
            },
            | _ => {
                let p = parent
                    .as_object_mut()
                    .unwrap()
                    .entry(&current_namespace)
                    .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                recursive_add(namespace, p, value);
            },
        }
    }

    let sel_chain = &config.chains[&chain];
    let mut hb = handlebars::Handlebars::new();
    hb.set_strict_mode(true);

    for mat in &sel_chain.matrix {
        let mut values_json = serde_json::Value::Object(serde_json::Map::new());
        for arg in &args {
            let namespaces_vec: Vec<String> = arg.0.split('.').map(|s| s.to_string()).collect();
            let mut namespaces = VecDeque::from(namespaces_vec);
            recursive_add(&mut namespaces, &mut values_json, arg.1);
        }

        for task in &sel_chain.tasks {
            for cmd in &task.run {
                let rendered_cmd = hb.render_template(cmd, &values_json)?;
                let final_cmd = if let Some(workdir) = &task.workdir {
                    format!("cd {} && {}", workdir, rendered_cmd)
                } else if let Some(workdir) = &mat.workdir {
                    format!("cd {} && {}", workdir, rendered_cmd)
                } else {
                    rendered_cmd.to_owned()
                };

                let mut envs_merged = HashMap::<&String, &String>::new();
                for source in vec![&config.env, &sel_chain.env, &mat.env, &task.env] {
                    if let Some(m) = source {
                        envs_merged.extend(m);
                    }
                }

                let mut cmd_proc = std::process::Command::new("sh");
                cmd_proc.envs(envs_merged);
                cmd_proc.arg("-c");
                cmd_proc.arg(&final_cmd);
                let cmd_exit_code = InteractiveProcess::new(cmd_proc, |l| match l {
                    | Ok(v) => println!("{}", v),
                    | Err(e) => println!("{}", e),
                })?
                .wait()?
                .code();
                if let Some(code) = cmd_exit_code {
                    if code != 0 {
                        return Err(Box::new(crate::error::ChildProcessError::new(&format!(
                            "command \"{}\" failed with code {}",
                            &final_cmd, code,
                        ))));
                    }
                }
            }
        }
    }

    Ok(())
}
