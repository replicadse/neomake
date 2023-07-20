use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("std")]
    Std(#[from] Box<dyn std::error::Error + Sync + Send>),
    #[error("io")]
    Io(#[from] std::io::Error),
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
    #[error("task node recursion")]
    NodeRecursion,
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
    #[error("serialize toml")]
    SerializeToml(#[from] toml::ser::Error),
    #[error("deserialize toml")]
    DeserializeToml(#[from] toml::de::Error),
    #[error("serialize ron")]
    SerializeRon(#[from] ron::error::Error),
    #[error("serialize ron")]
    DeserializeRon(#[from] ron::error::SpannedError),
    #[error("handlebars")]
    Handlebars(#[from] handlebars::RenderError),
}
