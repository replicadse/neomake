#![feature(hash_drain_filter)]

mod args;
mod config;
mod error;
mod output;

use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    error::Error,
    iter::FromIterator,
    result::Result,
    sync::{
        Arc,
        Mutex,
    },
};

use interactive_process::InteractiveProcess;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = args::ClapArgumentLoader::load()?;
    match args.command {
        | args::Command::Init => init().await,
        | args::Command::Run { config, chains, args } => run(config, chains, args).await,
        | args::Command::List { config, format } => list(config, format).await,
        | args::Command::Describe { config, chains, format } => describe(config, chains, format).await,
    }
}

async fn init() -> Result<(), Box<dyn Error>> {
    std::fs::write("./.neomake.yaml", crate::config::default_config())?;
    Ok(())
}

fn determine_order(
    chains: &HashMap<String, config::Chain>,
    entries: &Vec<String>,
) -> Result<Vec<HashSet<String>>, Box<dyn Error>> {
    let mut map = HashMap::<String, Vec<String>>::new();

    let mut seen = HashSet::<String>::new();
    let mut pending = VecDeque::<String>::new();
    pending.extend(entries.to_owned());

    while let Some(next) = pending.pop_back() {
        if seen.contains(&next) {
            continue;
        }
        seen.insert(next.clone());

        if let Some(pre) = &chains[&next].pre {
            map.insert(next, pre.clone());
            pending.extend(pre.clone());
        } else {
            map.insert(next, Vec::<String>::new());
        }
    }
    seen.clear();

    let mut result = Vec::<HashSet<String>>::new();
    while map.len() > 0 {
        let leafs = map
            .drain_filter(|_, v| {
                for v_item in v {
                    if !seen.contains(v_item) {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<_>>();
        if leafs.len() == 0 {
            return Err(Box::new(crate::error::TaskChainRecursion::new(
                "recursion in graph detected",
            )));
        }
        let set = leafs.iter().map(|x| x.0.clone());
        seen.extend(set.clone());
        result.push(HashSet::<String>::from_iter(set));
    }

    Ok(result)
}

async fn run(
    conf: crate::config::Config,
    chains: Vec<String>,
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

    fn execute_matrix_entry(
        conf: &config::Config,
        chain: &config::Chain,
        mat: &config::MatrixEntry,
        args: &HashMap<String, String>,
        output: Arc<Mutex<output::Controller>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut hb = handlebars::Handlebars::new();
        hb.set_strict_mode(true);
        let mut values_json = serde_json::Value::Object(serde_json::Map::new());
        for arg in args {
            let namespaces_vec: Vec<String> = arg.0.split('.').map(|s| s.to_string()).collect();
            let mut namespaces = VecDeque::from(namespaces_vec);
            recursive_add(&mut namespaces, &mut values_json, arg.1);
        }

        for task in &chain.tasks {
            let rendered_cmd = hb.render_template(&task.script, &values_json)?;

            // respect workdir from most inner to outer scope
            let workdir = if let Some(workdir) = &task.workdir {
                Some(workdir)
            } else if let Some(workdir) = &mat.workdir {
                Some(workdir)
            } else if let Some(workdir) = &chain.workdir {
                Some(workdir)
            } else {
                None
            };

            let mut envs_merged = HashMap::<&String, &String>::new();
            for env in vec![&conf.env, &chain.env, &mat.env, &task.env] {
                if let Some(m) = env {
                    envs_merged.extend(m);
                }
            }

            let shell = if let Some(shell) = &task.shell {
                shell.to_owned()
            } else if let Some(shell) = &chain.shell {
                shell.to_owned()
            } else {
                config::Shell {
                    program: "sh".to_owned(),
                    args: vec!["-c".to_owned()],
                }
            };

            let mut cmd_proc = std::process::Command::new(&shell.program);
            cmd_proc.args(shell.args);
            cmd_proc.envs(envs_merged);
            if let Some(w) = workdir {
                cmd_proc.current_dir(w);
            }
            cmd_proc.arg(&rendered_cmd);
            let closure_controller = output.clone();
            let cmd_exit_code = InteractiveProcess::new(cmd_proc, move |l| match l {
                | Ok(v) => {
                    let mut lock = closure_controller.lock().unwrap();
                    lock.append(v);
                    lock.draw().unwrap();
                },
                | Err(..) => {},
            })?
            .wait()?
            .code();
            if let Some(code) = cmd_exit_code {
                if code != 0 {
                    let err_msg = format!("command \"{}\" failed with code {}", &rendered_cmd, code,);
                    return Err(Box::new(crate::error::ChildProcessError::new(&err_msg)));
                }
            }
        }
        Ok(())
    }

    let output = Arc::new(Mutex::new(output::Controller::new("==> ".to_owned(), 10)));
    for stage_chains in determine_order(&conf.chains, &chains)? {
        for chain_name in stage_chains {
            let chain = &conf.chains[&chain_name];

            if let Some(matrix) = &chain.matrix {
                for mat in matrix {
                    execute_matrix_entry(&conf, chain, mat, &args, output.clone())?;
                }
            } else {
                execute_matrix_entry(
                    &conf,
                    chain,
                    &config::MatrixEntry { ..Default::default() },
                    &args,
                    output.clone(),
                )?;
            }
        }
    }
    Ok(())
}

async fn list(config: crate::config::Config, format: args::Format) -> Result<(), Box<dyn Error>> {
    #[derive(Debug, serde::Serialize)]
    struct Output {
        chains: Vec<OutputChain>,
    }
    #[derive(Debug, serde::Serialize)]
    struct OutputChain {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pre: Option<Vec<String>>,
    }

    let info = Output {
        chains: Vec::from_iter(config.chains.iter().map(|c| OutputChain {
            name: c.0.to_owned(),
            pre: c.1.pre.clone(),
        })),
    };
    match format {
        | args::Format::YAML => println!("{}", serde_yaml::to_string(&info)?),
        | args::Format::JSON => println!("{}", serde_json::to_string(&info)?),
    };

    Ok(())
}

async fn describe(
    config: crate::config::Config,
    chains: Vec<String>,
    format: args::Format,
) -> Result<(), Box<dyn Error>> {
    let structure = determine_order(&config.chains, &chains)?;

    #[derive(Debug, serde::Serialize)]
    struct Output {
        stages: Vec<Vec<String>>,
    }

    let mut info = Output { stages: Vec::new() };
    for s in structure {
        info.stages
            .push(s.iter().map(|s| s.to_owned()).into_iter().collect::<Vec<_>>());
    }

    match format {
        | args::Format::JSON => println!("{}", serde_json::to_string(&info)?),
        | args::Format::YAML => println!("{}", serde_yaml::to_string(&info)?),
    };

    Ok(())
}
