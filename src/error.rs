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
    #[cfg(feature = "format_toml")]
    #[error("serialize toml")]
    SerializeToml(#[from] toml::ser::Error),
    #[cfg(feature = "format_toml")]
    #[error("deserialize toml")]
    DeserializeToml(#[from] toml::de::Error),
    #[cfg(feature = "format_ron")]
    #[error("serialize ron")]
    SerializeRon(#[from] ron::error::Error),
    #[cfg(feature = "format_ron")]
    #[error("serialize ron")]
    DeserializeRon(#[from] ron::error::SpannedError),
    #[error("handlebars")]
    Handlebars(#[from] handlebars::RenderError),
    #[error("regex")]
    RegexError(#[from] fancy_regex::Error),
}
