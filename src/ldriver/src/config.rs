use semver::Version;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// deserialized representation of `L.toml`
#[derive(Debug, Deserialize)]
pub struct LConfig {
    /// path to `L.toml`
    crate toml_path: PathBuf,
    /// path to `main.l`
    crate main_path: PathBuf,
    crate toml: TomlConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct TomlConfig {
    package: PkgConfig,
}

impl LConfig {
    pub fn from_main_path(main_path: PathBuf) -> Self {
        Self { main_path, toml_path: Default::default(), toml: TomlConfig::default() }
    }
}

// this impl only used to running tests
impl Default for PkgConfig {
    fn default() -> Self {
        Self { name: Default::default(), version: Version::new(0, 0, 0) }
    }
}

#[derive(Debug, Deserialize)]
pub struct PkgConfig {
    name: String,
    version: Version,
}

crate fn load_config(path: &Path) -> io::Result<LConfig> {
    let path = path.canonicalize()?;
    let toml_path = match load_toml(&path)? {
        Some(toml) => toml,
        None => panic!(),
    };

    // the given path could be either `path/to/pkg/L.toml`
    // or `path/to/pkg`
    // we expect `main_path` to be `path/to/pkg/src/main.l`
    let content = fs::read_to_string(&toml_path)?;
    let toml = toml::de::from_str(&content)?;
    let mut main_path = toml_path.clone();
    main_path.pop();
    main_path.push("src/main.l");
    Ok(LConfig { toml, main_path, toml_path: toml_path.to_path_buf() })
}

fn load_toml(path: &Path) -> io::Result<Option<PathBuf>> {
    if path.is_dir() {
        for file in path.read_dir()? {
            let file = file?;
            if file.file_name().to_str().unwrap() == "L.toml" {
                let toml_path = file.path();
                return Ok(Some(toml_path));
            }
        }
        Ok(None)
    } else {
        Ok(Some(path.to_path_buf()))
    }
}
