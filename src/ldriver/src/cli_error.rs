macro_rules! impl_from {
    ($from:ty, $variant:ident) => {
        impl From<$from> for CliError {
            fn from(from: $from) -> Self {
                Self::$variant(from)
            }
        }
    };
}

impl_from!(std::io::Error, Io);

#[derive(Debug)]
pub enum CliError {
    Io(std::io::Error),
}
