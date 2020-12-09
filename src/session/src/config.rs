use clap::Clap;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clap, Deserialize)]
pub struct CompilerOptions {
    /// the path of either the directory holding `L.toml`
    /// or the path of the main `.l` file
    /// default to the current directory
    #[clap(default_value = ".")]
    pub input_path: PathBuf,
    // TODO take optimization level as parameter
}

impl CompilerOptions {
    pub fn new(input_path: PathBuf) -> Self {
        Self { input_path }
    }
}
