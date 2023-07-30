use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use interactive_process::InteractiveProcess;
use threadpool::ThreadPool;

use crate::{
    error::Error,
    output::{self, Controller},
    plan,
};

pub(crate) struct ExecutionEngine {
    pub output: Arc<Mutex<output::Controller>>,
}

impl ExecutionEngine {
    pub fn new(prefix: String, silent: bool) -> Self {
        Self {
            output: Arc::new(Mutex::new(Controller::new(
                !silent,
                prefix,
                Box::new(std::io::stdout()),
            ))),
        }
    }

    pub async fn execute(&self, plan: plan::ExecutionPlan, workers: usize) -> Result<()> {
        struct Work {
            workdir: Option<String>,
            env: HashMap<String, String>,
            shell: plan::Shell,
            command: String,
        }

        for stage in &plan.stages {
            let pool = ThreadPool::new(workers);
            let (signal_tx, signal_rx) = std::sync::mpsc::channel::<Result<()>>();
            let mut signal_cnt = 0;

            let nodes = stage.nodes.iter().map(|v| plan.nodes.get(v).unwrap());
            for node in nodes {
                for matrix in &node.invocations {
                    let t_tx = signal_tx.clone();
                    let t_output = self.output.clone();

                    let mut work = Vec::<Work>::new();

                    for task in &node.tasks {
                        let workdir = if let Some(workdir) = &task.workdir {
                            Some(workdir.to_owned())
                        } else if let Some(workdir) = &node.workdir {
                            Some(workdir.to_owned())
                        } else {
                            None
                        };

                        let shell = if let Some(shell) = &task.shell {
                            shell.to_owned()
                        } else if let Some(shell) = &node.shell {
                            shell.to_owned()
                        } else {
                            crate::plan::Shell {
                                program: "sh".to_owned(),
                                args: vec!["-c".to_owned()],
                            }
                        };

                        let mut env = plan.env.clone();
                        env.extend(node.env.clone());
                        env.extend(matrix.env.clone());
                        env.extend(task.env.clone());

                        signal_cnt += 1;
                        work.push(Work {
                            command: task.cmd.clone(),
                            env,
                            shell,
                            workdir,
                        })
                    }

                    // executes matrix entry
                    pool.execute(move || {
                        let res = move || -> Result<()> {
                            for w in work {
                                let mut cmd_proc = std::process::Command::new(w.shell.program);
                                cmd_proc.args(w.shell.args);
                                cmd_proc.envs(w.env);
                                if let Some(w) = w.workdir {
                                    cmd_proc.current_dir(w);
                                }
                                cmd_proc.arg(&w.command);

                                let loc_out = t_output.clone();
                                let exit_status = InteractiveProcess::new(&mut cmd_proc, move |l| match l {
                                    | Ok(v) => {
                                        let mut lock = loc_out.lock().unwrap();
                                        lock.print(&v).expect("could not print");
                                    },
                                    | Err(..) => {},
                                })?
                                .wait()?;
                                if let Some(code) = exit_status.code() {
                                    if code != 0 {
                                        let err_msg = format!("command \"{}\" failed with code {}", &w.command, code);
                                        return Err(Error::ChildProcess(err_msg).into());
                                    }
                                }
                            }
                            Ok(())
                        }();
                        match res {
                            | Ok(..) => t_tx.send(Ok(())).expect("send failed"),
                            | Err(e) => t_tx
                                // error formatting should be improved
                                .send(Err(Error::Generic(format!("{:?}", e)).into()))
                                .expect("send failed"),
                        }
                    });
                }
            }

            let errs = signal_rx
                .iter()
                .take(signal_cnt)
                .filter(|x| x.is_err())
                .map(|x| x.expect_err("expect"))
                .collect::<Vec<_>>();
            if errs.len() > 0 {
                return Err(Error::Many(errs).into()); // abort at this stage
            }
        }
        Ok(())
    }
}
