use std::{
    collections::HashMap,
    error::Error,
    iter::FromIterator,
    result::Result,
};

#[derive(Debug)]
pub struct CallArgs {
    pub experimental: bool,
    pub command: Command,
}

impl CallArgs {
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.experimental {
            return Ok(());
        }

        match &self.command {
            | Command::Describe { .. } => Err(Box::new(crate::error::ExperimentalCommandError::new(
                "command is experimental",
            ))),
            | Command::List { .. } => Err(Box::new(crate::error::ExperimentalCommandError::new(
                "command is experimental",
            ))),
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
        config: crate::config::Config,
        chains: Vec<String>,
        args: HashMap<String, String>,
    },
    List {
        config: crate::config::Config,
        format: Format,
    },
    Describe {
        config: crate::config::Config,
        chains: Vec<String>,
        format: Format,
    },
}

pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub fn load() -> Result<CallArgs, Box<dyn Error>> {
        let command = clap::App::new("neomake")
            .version(env!("CARGO_PKG_VERSION"))
            .about("neomake")
            .author("replicadse <aw@voidpointergroup.com>")
            .arg(
                clap::Arg::new("experimental")
                    .short('e')
                    .long("experimental")
                    .value_name("EXPERIMENTAL")
                    .help("Enables experimental features that do not count as stable.")
                    .required(false)
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
                            .value_name("CONFIG")
                            .help("The configuration file to use.")
                            .default_value("./.neomake.yaml")
                            .multiple_values(false)
                            .required(false)
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("chain")
                            .short('c')
                            .long("chain")
                            .value_name("CHAIN")
                            .help("Which chain to execute.")
                            .multiple_occurrences(true)
                            .required(true)
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("arg")
                            .short('a')
                            .long("arg")
                            .value_name("ARG")
                            .help("An argument to the chain.")
                            .multiple_values(true)
                            .required(false)
                            .takes_value(true),
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
                            .value_name("CONFIG")
                            .help("The configuration file to use.")
                            .default_value("./.neomake.yaml")
                            .multiple_values(false)
                            .required(false)
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("chain")
                            .short('c')
                            .long("chain")
                            .value_name("CHAIN")
                            .help("Which chain to execute.")
                            .multiple_occurrences(true)
                            .required(true)
                            .takes_value(true),
                    )
                    .arg(
                        clap::Arg::new("output")
                            .short('o')
                            .long("output")
                            .value_name("OUTPUT")
                            .help("The output format.")
                            .default_value("yaml")
                            .possible_values(&["yaml", "json"])
                            .required(false)
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
                            .value_name("CONFIG")
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
                            .value_name("OUTPUT")
                            .help("The output format.")
                            .default_value("yaml")
                            .possible_values(&["yaml", "json"])
                            .required(false)
                            .takes_value(true),
                    ),
            )
            .get_matches();

        fn parse_config(x: &clap::ArgMatches) -> Result<crate::config::Config, Box<dyn Error>> {
            let config_content = if x.is_present("config") {
                let config_param = x.value_of("config").unwrap();
                std::fs::read_to_string(config_param)?
            } else {
                return Err(Box::new(crate::error::MissingArgumentError::new(
                    "configuration has not been specified",
                )));
            };

            fn check_version(config: &str) -> Result<(), Box<dyn Error>> {
                #[derive(Debug, serde::Deserialize)]
                struct WithVersion {
                    version: String,
                }
                let v: WithVersion = serde_yaml::from_str(config)?;

                if v.version != "0.2" {
                    Err(Box::new(crate::error::VersionCompatibilityError::new(&format!(
                        "config version {} is incompatible with this CLI version",
                        v.version
                    ))))
                } else {
                    Ok(())
                }
            }
            check_version(&config_content)?;

            Ok(serde_yaml::from_str(&config_content)?)
        }

        fn parse_chains(x: &clap::ArgMatches) -> Result<Vec<String>, Box<dyn Error>> {
            let chains = x
                .values_of("chain")
                .ok_or(Box::new(crate::error::MissingArgumentError::new(
                    "chain was not specified",
                )))?;

            Ok(Vec::<String>::from_iter(chains.into_iter().map(|v| v.to_owned())))
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
                config: parse_config(x)?,
                chains: parse_chains(x)?,
                args: args_map,
            }
        } else if let Some(x) = command.subcommand_matches("list") {
            let format = if let Some(f) = x.value_of("output") {
                match f {
                    | "yaml" => Format::YAML,
                    | "json" => Format::JSON,
                    | _ => Err(Box::new(crate::error::ArgumentError::new("unkown output format")))?,
                }
            } else {
                Format::JSON
            };

            Command::List {
                config: parse_config(x)?,
                format,
            }
        } else if let Some(x) = command.subcommand_matches("describe") {
            let format = if let Some(f) = x.value_of("output") {
                match f {
                    | "yaml" => Format::YAML,
                    | "json" => Format::JSON,
                    | _ => Err(Box::new(crate::error::ArgumentError::new("unkown output format")))?,
                }
            } else {
                Format::JSON
            };

            Command::Describe {
                config: parse_config(x)?,
                chains: parse_chains(x)?,
                format,
            }
        } else {
            return Err(Box::new(crate::error::UnknownCommandError::new("unknown command")));
        };

        let callargs = CallArgs {
            experimental: command.contains_id("experimental"),
            command: cmd,
        };

        callargs.validate()?;
        Ok(callargs)
    }
}
