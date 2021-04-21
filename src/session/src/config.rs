use clap::Clap;
use error::ErrorFormat;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Clap, Deserialize)]
pub struct CompilerOptions {
    /// the path of either the directory holding `L.toml`
    /// or the path of the main `.l` file
    #[clap(default_value = ".")]
    pub input_path: PathBuf,
    #[clap(long("error-format"), default_value = "text")]
    pub error_format: ErrorFormat,
    // TODO take optimization level as parameter (or debug/release)
}

impl CompilerOptions {
    pub fn with_input_path(input_path: PathBuf) -> Self {
        Self { input_path, ..Self::default() }
    }
}
