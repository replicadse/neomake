use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    result::Result,
    str::FromStr,
};

use clap::{Arg, ArgAction};

use crate::error::Error;

#[derive(Debug, Eq, PartialEq)]
pub enum Privilege {
    Normal,
    Experimental,
}

#[derive(Debug)]
pub(crate) struct CallArgs {
    pub privileges: Privilege,
    pub command: Command,
}

impl CallArgs {
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.privileges == Privilege::Experimental {
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
pub enum ManualFormat {
    Manpages,
    Markdown,
}

#[derive(Debug)]
pub enum Format {
    JSON,
    YAML,
}

#[derive(Debug)]
pub enum Command {
    Manual {
        path: String,
        format: ManualFormat,
    },
    Autocomplete {
        path: String,
        shell: clap_complete::Shell,
    },
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
    pub fn root_command() -> clap::Command {
        clap::Command::new("neomake")
            .version(env!("CARGO_PKG_VERSION"))
            .about("A rusty text templating application for CLIs.")
            .author("replicadse <aw@voidpointergroup.com>")
            .propagate_version(true)
            .subcommand_required(true)
            .args([Arg::new("experimental")
                .short('e')
                .long("experimental")
                .help("enables experimental features")
                .num_args(0)])
            .subcommand(
                clap::Command::new("man")
                    .about("Renders the manual.")
                    .arg(clap::Arg::new("out").short('o').long("out").required(true))
                    .arg(
                        clap::Arg::new("format")
                            .short('f')
                            .long("format")
                            .value_parser(["manpages", "markdown"])
                            .required(true),
                    ),
            )
            .subcommand(
                clap::Command::new("autocomplete")
                    .about("Renders shell completion scripts.")
                    .arg(clap::Arg::new("out").short('o').long("out").required(true))
                    .arg(
                        clap::Arg::new("shell")
                            .short('s')
                            .long("shell")
                            .value_parser(["bash", "zsh", "fish", "elvish", "powershell"])
                            .required(true),
                    ),
            )
            .subcommand(
                clap::Command::new("init").about("Initializes a new default configuration in the current folder."),
            )
            .subcommand(
                clap::Command::new("run")
                    .about("Runs task chains.")
                    .visible_aliases(&["r", "exec", "x"])
                    .arg(
                        Arg::new("config")
                            .short('f')
                            .long("config")
                            .help("The configuration file to use.")
                            .default_value("./.neomake.yaml"),
                    )
                    .arg(
                        Arg::new("chain")
                            .short('c')
                            .long("chain")
                            .action(ArgAction::Append)
                            .help("Which chain to execute."),
                    )
                    .arg(
                        Arg::new("arg")
                            .short('a')
                            .long("arg")
                            .action(ArgAction::Append)
                            .help("An argument to the chain.")
                    )
                    .arg(
                        Arg::new("workers")
                            .short('w')
                            .long("workers")
                            .help("Defines how many worker threads are used for tasks that can be executed in parllel.")
                            .default_value("1"),
                    ),
            )
            .subcommand(
                clap::Command::new("describe")
                    .about("Describes the execution graph for a given task chain configuration.")
                    .visible_aliases(&["d", "desc"])
                    .arg(
                        Arg::new("config")
                            .short('f')
                            .long("config")
                            .help("The configuration file to use.")
                            .default_value("./.neomake.yaml"),
                    )
                    .arg(
                        Arg::new("chain")
                            .short('c')
                            .long("chain")
                            .action(ArgAction::Append)
                            .help("Which chain to execute."),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("The output format.")
                            .default_value("yaml")
                            .value_parser(["yaml", "json"]),
                    ),
            )
            .subcommand(
                clap::Command::new("list")
                    .about("Lists all available task chains.")
                    .visible_aliases(&["ls"])
                    .arg(
                        clap::Arg::new("config")
                            .short('f')
                            .long("config")
                            .help("The configuration file to use.")
                            .default_value("./.neomake.yaml"),
                    )
                    .arg(
                        Arg::new("output")
                            .short('o')
                            .long("output")
                            .help("The output format.")
                            .default_value("yaml")
                            .value_parser(["yaml", "json"]),
                    ),
            )
    }

    pub fn load() -> Result<CallArgs, Box<dyn std::error::Error>> {
        let command = Self::root_command().get_matches();

        let privileges = if command.get_flag("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        fn parse_chains(x: &clap::ArgMatches) -> Result<HashSet<String>, Error> {
            let chains = x
                .get_many::<String>("chain")
                .ok_or(Error::MissingArgument("chain".to_owned()))?;

            Ok(HashSet::<String>::from_iter(chains.into_iter().map(|v| v.to_owned())))
        }

        let cmd = if let Some(subc) = command.subcommand_matches("man") {
            Command::Manual {
                path: subc.get_one::<String>("out").unwrap().into(),
                format: match subc.get_one::<String>("format").unwrap().as_str() {
                    | "manpages" => ManualFormat::Manpages,
                    | "markdown" => ManualFormat::Markdown,
                    | _ => return Err(Box::new(Error::Argument("unknown format".into()))),
                },
            }
        } else if let Some(subc) = command.subcommand_matches("autocomplete") {
            Command::Autocomplete {
                path: subc.get_one::<String>("out").unwrap().into(),
                shell: clap_complete::Shell::from_str(subc.get_one::<String>("shell").unwrap().as_str()).unwrap(),
            }
        } else if let Some(..) = command.subcommand_matches("init") {
            Command::Init
        } else if let Some(x) = command.subcommand_matches("run") {
            let mut args_map: HashMap<String, String> = HashMap::new();
            if let Some(args) = x.get_many::<String>("arg") {
                for v_arg in args {
                    let spl: Vec<&str> = v_arg.splitn(2, "=").collect();
                    args_map.insert(spl[0].to_owned(), spl[1].to_owned());
                }
            }

            Command::Run {
                config: std::fs::read_to_string(x.get_one::<String>("config").unwrap())?,
                chains: parse_chains(x)?,
                args: args_map,
                workers: str::parse::<usize>(x.get_one::<String>("workers").unwrap())?,
            }
        } else if let Some(x) = command.subcommand_matches("list") {
            let format = match x.get_one::<String>("output").unwrap().as_str() {
                | "yaml" => Format::YAML,
                | "json" => Format::JSON,
                | _ => Err(Error::Argument("output".to_owned()))?,
            };

            Command::List {
                config: std::fs::read_to_string(x.get_one::<String>("config").unwrap())?,
                format,
            }
        } else if let Some(x) = command.subcommand_matches("describe") {
            let format = match x.get_one::<String>("output").unwrap().as_str() {
                | "yaml" => Format::YAML,
                | "json" => Format::JSON,
                | _ => Err(Error::Argument("output".to_owned()))?,
            };

            Command::Describe {
                config: std::fs::read_to_string(x.get_one::<String>("config").unwrap())?,
                chains: parse_chains(x)?,
                format,
            }
        } else {
            return Err(Box::new(Error::UnknownCommand));
        };

        let callargs = CallArgs {
            privileges,
            command: cmd,
        };

        callargs.validate()?;
        Ok(callargs)
    }
}
