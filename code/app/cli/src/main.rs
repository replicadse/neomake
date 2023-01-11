#![feature(hash_drain_filter)]

mod args;
mod config;
mod error;
mod model;
mod output;

use std::{
    error::Error,
    result::Result,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = args::ClapArgumentLoader::load()?;
    match args.command {
        | args::Command::Init => init().await,
        | args::Command::Run { config, chains, args } => {
            let m = model::Config::load_from_config(&config)?;
            m.execute(&chains, &args).await?;
            Ok(())
        },
        | args::Command::List { config, format } => {
            let m = model::Config::load_from_config(&config)?;
            m.list(format).await?;
            Ok(())
        },
        | args::Command::Describe { config, chains, format } => {
            let m = model::Config::load_from_config(&config)?;
            m.describe(chains, format).await?;
            Ok(())
        },
    }
}

async fn init() -> Result<(), Box<dyn Error>> {
    std::fs::write("./.neomake.yaml", crate::config::default_config())?;
    Ok(())
}
