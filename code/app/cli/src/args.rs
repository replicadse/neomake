use std::{
    collections::HashMap,
    error::Error,
    result::Result,
};

#[derive(Debug)]
pub struct CallArgs {
    pub command: Command,
}

impl CallArgs {
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

#[derive(Debug)]
/// The (sub-)command representation for the call args.
pub enum Command {
    Init,
    Run {
        config: crate::config::Config,
        chain: String,
        args: HashMap<String, String>,
    },
}

/// The type that parses the arguments to the program.
pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    /// Parsing the program arguments with the `clap` trait.
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
            .subcommand(clap::App::new("init").about(""))
            .subcommand(
                clap::App::new("run")
                    .about("")
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
                            .multiple_values(false)
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
            .get_matches();

        let cmd = if let Some(..) = command.subcommand_matches("init") {
            Command::Init
        } else if let Some(x) = command.subcommand_matches("run") {
            let chain = x
                .value_of("chain")
                .ok_or(Box::new(crate::error::MissingArgumentError::new(
                    "chain was not specified",
                )))?;

            // parse args
            let mut args_map: HashMap<String, String> = HashMap::new();
            if let Some(args) = x.values_of("arg") {
                for v_arg in args {
                    let spl: Vec<&str> = v_arg.splitn(2, "=").collect();
                    args_map.insert(spl[0].to_owned(), spl[1].to_owned());
                }
            }

            // parse config
            let config_content = if x.is_present("config") {
                let config_param = x.value_of("config").unwrap();
                std::fs::read_to_string(config_param)?
            } else {
                return Err(Box::new(crate::error::MissingArgumentError::new(
                    "configuration has not been specified",
                )));
            };
            Command::Run {
                chain: chain.to_owned(),
                args: args_map,
                config: serde_yaml::from_str(&config_content)?,
            }
        } else {
            return Err(Box::new(crate::error::UnknownCommandError::new("unknown command")));
        };

        let callargs = CallArgs { command: cmd };

        callargs.validate()?;
        Ok(callargs)
    }
}
