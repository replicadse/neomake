use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("std")]
    Std(#[from] Box<dyn std::error::Error + Sync + Send>),
    #[error("generic")]
    Generic(String),
    #[error("many")]
    Many(Vec<Self>),

    // #[error("experimental command")]
    // ExperimentalCommand,
    #[error("argument")]
    Argument(String),
    #[error("missing argument")]
    MissingArgument(String),
    #[error("child process")]
    ChildProcess(String),
    #[error("task chain recursion")]
    TaskChainRecursion,
    #[error("unknown command")]
    UnknownCommand,
    #[error("version compatibility")]
    VersionCompatibility(String),
    #[error("not found")]
    NotFound(String),
    #[error("serde json")]
    SerdeJson(#[from] serde_json::Error),
    #[error("serde yaml")]
    SerdeYaml(#[from] serde_yaml::Error),
}
