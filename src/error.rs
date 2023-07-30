#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("generic")]
    Generic(String),
    #[error("many")]
    Many(Vec<anyhow::Error>),

    // ExperimentalCommand,
    #[error("argument")]
    Argument(String),
    #[error("child process")]
    ChildProcess(String),
    #[error("node recursion")]
    NodeRecursion,
    #[error("unknown command")]
    UnknownCommand,
    #[error("generic")]
    VersionCompatibility(String),
    #[error("not found")]
    NotFound(String),
}
