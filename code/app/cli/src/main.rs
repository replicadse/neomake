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

    let selected_chain = &config.chains[&chain];
    let mut hb = handlebars::Handlebars::new();
    hb.set_strict_mode(true);

    for matrix in &selected_chain.matrix {
        let mut values_json = serde_json::Value::Object(serde_json::Map::new());
        for arg in &args {
            let namespaces_vec: Vec<String> = arg.0.split('.').map(|s| s.to_string()).collect();
            let mut namespaces = VecDeque::from(namespaces_vec);
            recursive_add(&mut namespaces, &mut values_json, arg.1);
        }

        let mut rendered_commands = Vec::<String>::new();
        for task in &selected_chain.tasks {
            for cmd in &task.run {
                rendered_commands.push(hb.render_template(cmd, &values_json)?)
            }
        }

        for cmd in rendered_commands {
            let final_cmd = if let Some(workdir) = &matrix.workdir {
                format!("cd {} && {}", workdir, cmd)
            } else {
                cmd
            };
            let cmd_out = std::process::Command::new("sh")
                .envs(&matrix.env)
                .arg("-c")
                .arg(final_cmd)
                .output()?;
            if cmd_out.stderr.len() > 0 {
                println!("{}", String::from_utf8(cmd_out.stderr)?);
            }
            if cmd_out.stdout.len() > 0 {
                println!("{}", String::from_utf8(cmd_out.stdout)?);
            }
        }
    }

    Ok(())
}
