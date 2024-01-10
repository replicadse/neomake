#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    // #[error("generic {0}")]
    // Generic(String),
    #[error("many: {0:?}")]
    Many(Vec<anyhow::Error>),
    #[error("experimental command: {0}")]
    ExperimentalCommand(String),
    #[error("argument {0}")]
    Argument(String),
    #[error("child process {0}")]
    ChildProcess(String),
    #[error("node recursion")]
    NodeRecursion,
    #[error("unknown command")]
    UnknownCommand,
    #[error("version compatibility {0}")]
    VersionCompatibility(String),
    #[error("not found {0}")]
    NotFound(String),
}
