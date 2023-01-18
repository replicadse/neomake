use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    result::Result,
};

use crate::error::Error;

#[derive(Debug)]
pub(crate) struct CallArgs {
    pub experimental: bool,
    pub command: Command,
}

impl CallArgs {
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.experimental {
            return Ok(());
        }

        match &self.command {
            | Command::Describe { .. } => Err(Box::new(Error::ExperimentalCommand)),
            | Command::List { .. } => Err(Box::new(Error::ExperimentalCommand)),
            | _ => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum Format {
    JSON,
    YAML,
}

#[derive(Debug)]
pub enum Command {
    Init,
    Run {
        config: String,
        chains: HashSet<String>,
        args: HashMap<String, String>,
        workers: usize,
    },
    List {
        config: String,
        format: Format,
    },
    Describe {
        config: String,
        chains: HashSet<String>,
        format: Format,
    },
}

pub(crate) struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub fn load() -> Result<CallArgs, Box<dyn std::error::Error>> {
        let command = clap::App::new("neomake")
            .version(env!("CARGO_PKG_VERSION"))
            .about("neomake")
            .author("replicadse <aw@voidpointergroup.com>")
            .arg(
                clap::Arg::new("experimental")
                    .short('e')
                    .long("experimental")
                    .help("Enables experimental features that do not count as stable.")
                    .takes_value(false),
            )
            .subcommand(clap::App::new("init").about("Initializes a new default configuration in the current folder."))
            .subcommand(
                clap::App::new("run")
                    .about("Runs task chains.")
                    .visible_aliases(&["r", "exec", "x"])
                    .arg(
                        clap::Arg::new("config")
                            .short('f')
                            .long("config")
                            .help("The configuration file to use.")
                            .default_value("./.neomake.yaml")
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("chain")
                            .short('c')
                            .long("chain")
                            .help("Which chain to execute.")
                            .multiple_occurrences(true)
                            .required(true)
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("arg")
                            .short('a')
                            .long("arg")
                            .help("An argument to the chain.")
                            .multiple_occurrences(true)
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("workers")
                            .short('w')
                            .long("workers")
                            .help("Defines how many worker threads are used for tasks that can be executed in parllel.")
                            .takes_value(true)
                            .default_value("1"),
                    ),
            )
            .subcommand(
                clap::App::new("describe")
                    .about("Describes the execution graph for a given task chain configuration.")
                    .visible_aliases(&["d", "desc"])
                    .arg(
                        clap::Arg::new("config")
                            .short('f')
                            .long("config")
                            .help("The configuration file to use.")
                            .default_value("./.neomake.yaml")
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("chain")
                            .short('c')
                            .long("chain")
                            .help("Which chain to execute.")
                            .multiple_occurrences(true)
                            .required(true)
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("The output format.")
                            .default_value("yaml")
                            .possible_values(&["yaml", "json"])
                            .takes_value(true),
                    ),
            )
            .subcommand(
                clap::App::new("list")
                    .about("Lists all available task chains.")
                    .visible_aliases(&["ls"])
                    .arg(
                        clap::Arg::new("config")
                            .short('f')
                            .long("config")
                            .help("The configuration file to use.")
                            .default_value("./.neomake.yaml")
                            .multiple_values(false)
                            .required(false)
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("The output format.")
                            .default_value("yaml")
                            .possible_values(&["yaml", "json"])
                            .required(false)
                            .takes_value(true),
                    ),
            )
            .get_matches();

        fn parse_chains(x: &clap::ArgMatches) -> Result<HashSet<String>, Error> {
            let chains = x.values_of("chain").ok_or(Error::MissingArgument("chain".to_owned()))?;

            Ok(HashSet::<String>::from_iter(chains.into_iter().map(|v| v.to_owned())))
        }

        let cmd = if let Some(..) = command.subcommand_matches("init") {
            Command::Init
        } else if let Some(x) = command.subcommand_matches("run") {
            let mut args_map: HashMap<String, String> = HashMap::new();
            if let Some(args) = x.values_of("arg") {
                for v_arg in args {
                    let spl: Vec<&str> = v_arg.splitn(2, "=").collect();
                    args_map.insert(spl[0].to_owned(), spl[1].to_owned());
                }
            }

            Command::Run {
                config: std::fs::read_to_string(x.value_of("config").unwrap())?,
                chains: parse_chains(x)?,
                args: args_map,
                workers: str::parse::<usize>(x.value_of("workers").unwrap())?,
            }
        } else if let Some(x) = command.subcommand_matches("list") {
            let format = if let Some(f) = x.value_of("output") {
                match f {
                    | "yaml" => Format::YAML,
                    | "json" => Format::JSON,
                    | _ => Err(Error::Argument("output".to_owned()))?,
                }
            } else {
                Format::JSON
            };

            Command::List {
                config: std::fs::read_to_string(x.value_of("config").unwrap())?,
                format,
            }
        } else if let Some(x) = command.subcommand_matches("describe") {
            let format = if let Some(f) = x.value_of("output") {
                match f {
                    | "yaml" => Format::YAML,
                    | "json" => Format::JSON,
                    | _ => Err(Error::Argument("output".to_owned()))?,
                }
            } else {
                Format::JSON
            };

            Command::Describe {
                config: std::fs::read_to_string(x.value_of("config").unwrap())?,
                chains: parse_chains(x)?,
                format,
            }
        } else {
            return Err(Box::new(Error::UnknownCommand));
        };

        let callargs = CallArgs {
            experimental: command.contains_id("experimental"),
            command: cmd,
        };

        callargs.validate()?;
        Ok(callargs)
    }
}
