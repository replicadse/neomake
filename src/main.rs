include!("check_features.rs");

use std::path::PathBuf;
use std::result::Result;

use args::ManualFormat;

pub mod args;
pub mod config;
pub mod error;
pub mod model;
pub mod output;
pub mod reference;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = crate::args::ClapArgumentLoader::load()?;
    cmd.validate()?;

    match cmd.command {
        | crate::args::Command::Manual { path, format } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            match format {
                | ManualFormat::Manpages => {
                    reference::build_manpages(&out_path)?;
                },
                | ManualFormat::Markdown => {
                    reference::build_markdown(&out_path)?;
                },
            }
            Ok(())
        },
        | crate::args::Command::Autocomplete { path, shell } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            reference::build_shell_completion(&out_path, &shell)?;
            Ok(())
        },
        | crate::args::Command::Init => {
            std::fs::write("./.neomake.yaml", crate::config::default_config())?;
            Ok(())
        },
        | crate::args::Command::Run {
            config,
            chains,
            args,
            workers,
        } => {
            let m = model::Config::load_from_str(&config)?;
            m.execute(&chains, &args, workers).await?;
            Ok(())
        },
        | crate::args::Command::List { config, format } => {
            let m = model::Config::load_from_str(&config)?;
            m.list(format).await?;
            Ok(())
        },
        | crate::args::Command::Describe { config, chains, format } => {
            let m = model::Config::load_from_str(&config)?;
            m.describe(chains, format).await?;
            Ok(())
        },
    }
}
