#[macro_export]
macro_rules! make_error {
    ($name:ident) => {
        #[derive(Debug, Clone)]
        /// An error type.
        pub struct $name {
            details: String,
        }

        impl $name {
            /// Error type constructor.
            pub fn new(details: &str) -> Self {
                Self {
                    details: details.to_owned(),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.details)
            }
        }

        impl std::error::Error for $name {}
    };
}

#[macro_export]
macro_rules! make_error_enum {
    ($name:ident, $($en:ident),*) => {
        #[derive(Debug, Clone)]
        /// An error type.
        pub enum $name {
            $($en (String),)*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( Self::$en(e) => {
                        f.write_str(e)
                    }, )*
                }
            }
        }

        impl std::error::Error for $name {}
    };
}
