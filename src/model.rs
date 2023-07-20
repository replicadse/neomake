use std::{
    collections::{HashMap, HashSet, VecDeque},
    iter::FromIterator,
    sync::{Arc, Mutex},
};

use interactive_process::InteractiveProcess;
use itertools::Itertools;
use threadpool::ThreadPool;

use crate::{
    error::Error,
    output::{self, Controller},
    workflow::Shell,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Task {
    cmd: String,
    env: HashMap<String, String>,
    workdir: Option<String>,
    shell: Shell,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Matrix {
    tasks: Vec<Task>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Node {
    matrix: Vec<Matrix>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Stage {
    nodes: Vec<Node>,
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct ExecutionPlan {
    stages: Vec<Stage>,
}

pub(crate) struct Workflow {
    pub nodes: HashMap<String, crate::workflow::Node>,
    pub env: HashMap<String, String>,
}

impl Workflow {
    pub fn load(data: &str) -> Result<Self, Error> {
        #[derive(Debug, serde::Deserialize)]
        struct Versioned {
            version: String,
        }
        let v = serde_yaml::from_str::<Versioned>(data)?;

        if v.version != "0.5" {
            Err(Error::VersionCompatibility(format!(
                "workflow version {} is incompatible with this CLI version {}",
                v.version,
                env!("CARGO_PKG_VERSION")
            )))?
        }

        let cfg: crate::workflow::Workflow = serde_yaml::from_str(&data)?;
        Ok(Self {
            nodes: cfg.nodes,
            env: if let Some(e) = cfg.env {
                e
            } else {
                HashMap::<String, String>::new()
            },
        })
    }

    pub async fn render_exec(
        &self,
        nodes: &HashSet<String>,
        args: &HashMap<String, String>,
    ) -> Result<ExecutionPlan, Error> {
        let mut hb = handlebars::Handlebars::new();
        hb.set_strict_mode(true);
        let arg_vals = self.build_args(args)?;
        let stages = self.determine_order(nodes)?;

        let mut plan = ExecutionPlan { stages: vec![] };

        for stage in stages {
            let mut rendered_stage = Stage { nodes: vec![] };
            for node in stage {
                let mut rendered_node = Node { matrix: vec![] }; // nodes + tasks -> parallelize on l0
                let node_def = &self.nodes[&node];

                let matrix_entry_default = crate::workflow::MatrixCell { ..Default::default() };

                let matrix_cp = if let Some(matrix) = &node_def.matrix {
                    matrix.iter().multi_cartesian_product().collect::<Vec<_>>()
                } else {
                    vec![vec![&matrix_entry_default]]
                };

                for mat in matrix_cp {
                    let mut rendered_matrix = Matrix { tasks: vec![] };
                    for task in &node_def.tasks {
                        let rendered_cmd = hb.render_template(&task.script, &arg_vals)?;

                        let workdir = if let Some(workdir) = &task.workdir {
                            Some(workdir.to_owned())
                        } else if let Some(workdir) = &node_def.workdir {
                            Some(workdir.to_owned())
                        } else {
                            None
                        };

                        let shell = if let Some(shell) = &task.shell {
                            shell.to_owned()
                        } else if let Some(shell) = &node_def.shell {
                            shell.to_owned()
                        } else {
                            crate::workflow::Shell {
                                program: "sh".to_owned(),
                                args: vec!["-c".to_owned()],
                            }
                        };

                        let mut rendered_task = Task {
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
                        for env in vec![&self_env, &node_def.env, &combined_matrix_env, &task.env] {
                            if let Some(m) = env {
                                rendered_task.env.extend(m.clone());
                            }
                        }

                        rendered_matrix.tasks.push(rendered_task);
                    }
                    rendered_node.matrix.push(rendered_matrix);
                }
                rendered_stage.nodes.push(rendered_node);
            }
            plan.stages.push(rendered_stage);
        }

        Ok(plan)
    }

    pub async fn list(&self, format: &crate::args::Format) -> Result<(), Error> {
        #[derive(Debug, serde::Serialize)]
        struct Output {
            nodes: Vec<OutputNode>,
        }
        #[derive(Debug, serde::Serialize)]
        struct OutputNode {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            pre: Option<Vec<String>>,
        }

        let mut info = Output {
            nodes: Vec::from_iter(self.nodes.iter().map(|c| OutputNode {
                name: c.0.to_owned(),
                description: c.1.description.clone(),
                pre: c.1.pre.clone(),
            })),
        };
        info.nodes.sort_by(|a, b| a.name.cmp(&b.name));

        println!("{}", format.serialize(&info)?);

        Ok(())
    }

    pub async fn plan(&self, nodes: &HashSet<String>, format: &crate::args::Format) -> Result<(), Error> {
        let structure = self.determine_order(&nodes)?;

        #[derive(Debug, serde::Serialize)]
        struct Output {
            stages: Vec<Vec<String>>,
        }

        let mut info = Output { stages: Vec::new() };
        for s in structure {
            info.stages
                .push(s.iter().map(|s| s.to_owned()).into_iter().collect::<Vec<_>>());
        }

        println!("{}", format.serialize(&info)?);

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

            let c = self.nodes.get(&next);
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
                return Err(Error::NodeRecursion);
            }
            let set = leafs.iter().map(|x| x.0.clone());
            seen.extend(set.clone());
            result.push(HashSet::<String>::from_iter(set));
        }

        Ok(result)
    }
}

pub(crate) struct ExecEngine {
    pub output: Arc<Mutex<output::Controller>>,
}

impl ExecEngine {
    pub fn new(prefix: String, silent: bool) -> Self {
        Self {
            output: Arc::new(Mutex::new(Controller::new(
                !silent,
                prefix,
                Box::new(std::io::stdout()),
            ))),
        }
    }

    pub async fn execute(&self, plan: ExecutionPlan, workers: usize) -> Result<(), Error> {
        for stage in plan.stages {
            let signal_cnt = stage.nodes.iter().map(|c| c.matrix.len()).sum();
            let pool = ThreadPool::new(workers);
            let (tx, rx) = std::sync::mpsc::channel::<Result<(), Error>>();

            for node in stage.nodes {
                for matrix in node.matrix {
                    let output_thread = self.output.clone();
                    let tx_thread = tx.clone();

                    // executes matrix entry
                    pool.execute(move || {
                        let res = move || -> Result<(), Box<dyn std::error::Error>> {
                            for task in matrix.tasks {
                                let mut cmd_proc = std::process::Command::new(&task.shell.program);
                                cmd_proc.args(task.shell.args);
                                cmd_proc.envs(task.env);
                                if let Some(w) = task.workdir {
                                    cmd_proc.current_dir(w);
                                }
                                cmd_proc.arg(&task.cmd);

                                let loc_out = output_thread.clone();
                                let exit_status = InteractiveProcess::new(cmd_proc, move |l| match l {
                                    | Ok(v) => {
                                        let mut lock = loc_out.lock().unwrap();
                                        lock.print(&v).expect("could not print");
                                    },
                                    | Err(..) => {},
                                })?
                                .wait()?;
                                if let Some(code) = exit_status.code() {
                                    if code != 0 {
                                        let err_msg = format!("command \"{}\" failed with code {}", &task.cmd, code);
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
            }
            let errs = rx
                .iter()
                .take(signal_cnt)
                .filter(|x| x.is_err())
                .map(|x| x.expect_err("expect"))
                .collect::<Vec<_>>();
            if errs.len() > 0 {
                return Err(Error::Many(errs)); // abort at this stage
            }
        }
        Ok(())
    }
}
