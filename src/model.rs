use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter::FromIterator,
    sync::{Arc, Mutex},
};

use interactive_process::InteractiveProcess;
use itertools::Itertools;
use threadpool::ThreadPool;

use crate::{config::Shell, error::Error, output};

struct ExecVars {
    cmd: String,
    env: HashMap<String, String>,
    workdir: Option<String>,
    shell: Shell,
}

pub(crate) struct Config {
    pub output: Arc<Mutex<output::Controller>>,
    pub chains: HashMap<String, crate::config::Chain>,
    pub env: HashMap<String, String>,
}

impl Config {
    pub fn load_from_str(data: &str) -> Result<Self, Box<dyn std::error::Error>> {
        #[derive(Debug, serde::Deserialize)]
        struct WithVersion {
            version: String,
        }
        let v: WithVersion = serde_yaml::from_str(data)?;

        if v.version != "0.3" {
            Err(Error::VersionCompatibility(format!(
                "config version {:?} is incompatible with this CLI version",
                v
            )))?
        }

        let cfg: crate::config::Config = serde_yaml::from_str(&data)?;
        Ok(Self {
            output: Arc::new(Mutex::new(output::Controller::new("==> ".to_owned()))),
            chains: cfg.chains,
            env: if let Some(e) = cfg.env {
                e
            } else {
                HashMap::<String, String>::new()
            },
        })
    }

    pub async fn execute(
        &self,
        exec_chains: &HashSet<String>,
        args: &HashMap<String, String>,
        workers: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut hb = handlebars::Handlebars::new();
        hb.set_strict_mode(true);
        let arg_vals = self.build_args(args)?;

        let stages = self.determine_order(exec_chains)?;

        for stage in stages {
            let pool = ThreadPool::new(workers);
            let (tx, rx) = std::sync::mpsc::channel::<Result<(), Error>>();

            let mut execs = Vec::<Vec<ExecVars>>::new(); // chains + tasks -> parallelize on l0

            for tcn in stage {
                let tc = &self.chains[&tcn];

                let matrix_entry_default = crate::config::MatrixEntry { ..Default::default() };

                let matrix_cp = if let Some(matrix) = &tc.matrix {
                    matrix.iter().multi_cartesian_product().collect::<Vec<_>>()
                } else {
                    vec![vec![&matrix_entry_default]]
                };

                for mat in matrix_cp {
                    let mut task_execs = Vec::<ExecVars>::new();
                    for task in &tc.tasks {
                        let rendered_cmd = hb.render_template(&task.script, &arg_vals)?;

                        let workdir = if let Some(workdir) = &task.workdir {
                            Some(workdir.to_owned())
                        } else if let Some(workdir) = &tc.workdir {
                            Some(workdir.to_owned())
                        } else {
                            None
                        };

                        let shell = if let Some(shell) = &task.shell {
                            shell.to_owned()
                        } else if let Some(shell) = &tc.shell {
                            shell.to_owned()
                        } else {
                            crate::config::Shell {
                                program: "sh".to_owned(),
                                args: vec!["-c".to_owned()],
                            }
                        };

                        let mut exec_vars = ExecVars {
                            cmd: rendered_cmd,
                            env: HashMap::<String, String>::new(),
                            workdir,
                            shell,
                        };

                        let mut combined_matrix_env = Some(HashMap::<String, String>::new());
                        for i in 0..mat.len() {
                            if let Some(env_current) = &mat[i].env {
                                combined_matrix_env.as_mut().unwrap().extend(env_current.clone());
                            }
                        }

                        let self_env = Some(self.env.clone());
                        for env in vec![&self_env, &tc.env, &combined_matrix_env, &task.env] {
                            if let Some(m) = env {
                                exec_vars.env.extend(m.clone());
                            }
                        }

                        task_execs.push(exec_vars);
                    }
                    execs.push(task_execs);
                }
            }

            let signal_cnt = execs.len();
            for e in execs {
                let output_thread = self.output.clone();
                let tx_thread = tx.clone();
                pool.execute(move || {
                    let res = move || -> Result<(), Box<dyn std::error::Error>> {
                        for exec_vars in e {
                            let mut cmd_proc = std::process::Command::new(&exec_vars.shell.program);
                            cmd_proc.args(exec_vars.shell.args);
                            cmd_proc.envs(exec_vars.env);
                            if let Some(w) = exec_vars.workdir {
                                cmd_proc.current_dir(w);
                            }
                            cmd_proc.arg(&exec_vars.cmd);

                            let loc_out = output_thread.clone();
                            let exit_status = InteractiveProcess::new(cmd_proc, move |l| match l {
                                | Ok(v) => {
                                    let mut lock = loc_out.lock().unwrap();
                                    lock.append(v);
                                    lock.draw().unwrap();
                                },
                                | Err(..) => {},
                            })?
                            .wait()?;
                            if let Some(code) = exit_status.code() {
                                if code != 0 {
                                    let err_msg = format!("command \"{}\" failed with code {}", &exec_vars.cmd, code,);
                                    return Err(Box::new(Error::ChildProcess(err_msg)));
                                }
                            }
                        }
                        Ok(())
                    }();
                    match res {
                        | Ok(..) => tx_thread.send(Ok(())).expect("send failed"),
                        | Err(e) => tx_thread
                            // error formatting should be improved
                            .send(Err(Error::Generic(format!("{:?}", e))))
                            .expect("send failed"),
                    }
                });
            }
            let errs = rx
                .iter()
                .take(signal_cnt)
                .filter(|x| x.is_err())
                .map(|x| x.expect_err("expect"))
                .collect::<Vec<_>>();
            if errs.len() > 0 {
                return Err(Box::new(Error::Many(errs)));
            }
        }
        Ok(())
    }

    pub async fn list(&self, format: crate::args::Format) -> Result<(), Box<dyn std::error::Error>> {
        #[derive(Debug, serde::Serialize)]
        struct Output {
            chains: Vec<OutputChain>,
        }
        #[derive(Debug, serde::Serialize)]
        struct OutputChain {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pre: Option<Vec<String>>,
        }

        let mut info = Output {
            chains: Vec::from_iter(self.chains.iter().map(|c| OutputChain {
                name: c.0.to_owned(),
                description: c.1.description.clone(),
                pre: c.1.pre.clone(),
            })),
        };
        info.chains.sort_by(|a, b| a.name.cmp(&b.name));

        match format {
            | crate::args::Format::YAML => println!("{}", serde_yaml::to_string(&info)?),
            | crate::args::Format::JSON => println!("{}", serde_json::to_string(&info)?),
        };

        Ok(())
    }

    pub async fn describe(
        &self,
        chains: HashSet<String>,
        format: crate::args::Format,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let structure = self.determine_order(&chains)?;

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
            | crate::args::Format::JSON => println!("{}", serde_json::to_string(&info)?),
            | crate::args::Format::YAML => println!("{}", serde_yaml::to_string(&info)?),
        };

        Ok(())
    }

    fn build_args(&self, args: &HashMap<String, String>) -> Result<serde_json::Value, Error> {
        fn recursive_add(
            namespace: &mut std::collections::VecDeque<String>,
            parent: &mut serde_json::Value,
            value: &str,
        ) {
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
        let mut values_json = serde_json::Value::Object(serde_json::Map::new());
        for arg in args {
            let namespaces_vec: Vec<String> = arg.0.split('.').map(|s| s.to_string()).collect();
            let mut namespaces = VecDeque::from(namespaces_vec);
            recursive_add(&mut namespaces, &mut values_json, arg.1);
        }
        Ok(values_json)
    }

    fn determine_order(&self, exec: &HashSet<String>) -> Result<Vec<HashSet<String>>, Error> {
        let mut map = HashMap::<String, Vec<String>>::new();

        let mut seen = HashSet::<String>::new();
        let mut pending = VecDeque::<String>::new();
        pending.extend(exec.to_owned());

        while let Some(next) = pending.pop_back() {
            if seen.contains(&next) {
                continue;
            }
            seen.insert(next.clone());

            let c = self.chains.get(&next);
            if c.is_none() {
                return Err(Error::NotFound(next.to_owned()));
            }

            if let Some(pre) = &c.unwrap().pre {
                map.insert(next, pre.clone());
                pending.extend(pre.clone());
            } else {
                map.insert(next, Vec::<String>::new());
            }
        }
        seen.clear();

        let mut result = Vec::<HashSet<String>>::new();
        while map.len() > 0 {
            // This implementation SHOULD make use of the unstable hash_drain_filter feature
            // to use the drain_filter method on the hashmap but it's not allowed on stable yet.
            let leafs = map
                .iter()
                .filter_map(|(k, v)| {
                    for v_item in v {
                        if !seen.contains(v_item) {
                            return None;
                        }
                    }
                    Some((k.clone(), v.clone()))
                })
                .collect::<Vec<_>>();
            for v in &leafs {
                map.remove(&v.0);
            }

            if leafs.len() == 0 {
                return Err(Error::TaskChainRecursion);
            }
            let set = leafs.iter().map(|x| x.0.clone());
            seen.extend(set.clone());
            result.push(HashSet::<String>::from_iter(set));
        }

        Ok(result)
    }
}
