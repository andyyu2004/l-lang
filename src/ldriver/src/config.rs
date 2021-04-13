use semver::Version;
use serde::de::{self, Deserialize};
use session::CompilerOptions;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

/// deserialized representation of `L.toml`
#[derive(Debug, Deserialize)]
pub struct LConfig {
    /// root path of the project itself
    /// i.e. the parent of the `L.toml`
    crate root_path: PathBuf,
    crate toml: TomlConfig,
    crate opts: CompilerOptions,
}

crate fn load_config(opts: CompilerOptions) -> io::Result<LConfig> {
    let path = opts
        .input_path
        .canonicalize()
        .unwrap_or_else(|_| panic!("path `{}` does not exist", opts.input_path.display()));

    // if `path` is a directory we search it for a `L.toml` file and load the config using that
    let mut config = if path.is_dir() {
        let toml_path = match load_toml(&path)? {
            Some(toml) => toml,
            None => panic!("`L.toml` not found in `{}`", path.display()),
        };
        // the given path could be either `path/to/pkg/L.toml` or `path/to/pkg`
        let content = fs::read_to_string(&toml_path)?;
        // dbg!(toml::de::from_str::<toml::Value>(&content).unwrap());
        let toml = toml::de::from_str(&content)?;
        let mut main_path = toml_path.clone();
        main_path.pop();

        LConfig {
            toml,
            root_path: toml_path.parent().unwrap().to_path_buf(),
            opts: CompilerOptions::default(),
        }
    } else {
        // if `path` is a file, we just run that file
        LConfig::from_main_path(path.to_path_buf())
    };

    config.opts = opts;
    config.validate()?;
    Ok(config)
}

impl LConfig {
    pub fn validate(&self) -> io::Result<()> {
        for dep in self.dependencies.values() {
            match dep {
                Dependency::Simple(version) =>
                    if let Err(err) = Version::parse(version) {
                        panic!("{}", err)
                    },
                // check the dependencies exist
                Dependency::Detailed(info) =>
                    if let Some(path) = &info.path {
                        let dep_path = Path::new(&path);
                        let joined_path = self.root_path.join(dep_path);
                        self::load_config(CompilerOptions {
                            input_path: joined_path,
                            ..self.opts.clone()
                        })?;
                    },
            }
        }
        Ok(())
    }
}

impl Deref for LConfig {
    type Target = TomlConfig;

    fn deref(&self) -> &Self::Target {
        &self.toml
    }
}

impl DerefMut for LConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.toml
    }
}

impl LConfig {
    /// create a config with the main set to the given parameter
    /// used to run the driver on a test source file
    pub fn from_main_path(main_path: PathBuf) -> Self {
        let mut lcfg = Self {
            opts: CompilerOptions::with_input_path(main_path.clone()),
            toml: TomlConfig::default(),
            root_path: PathBuf::default(),
        };
        lcfg.bin.main_path = main_path;
        lcfg
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct TomlConfig {
    pub package: PkgConfig,
    #[serde(default = "Dependencies::default")]
    pub dependencies: Dependencies,
    #[serde(default = "BinConfig::default")]
    pub bin: BinConfig,
}

#[derive(Debug, Deserialize)]
pub struct BinConfig {
    /// path of the `main` file relative to
    #[serde(default = "default_main_file")]
    pub main_path: PathBuf,
}

impl Default for BinConfig {
    fn default() -> Self {
        Self { main_path: default_main_file() }
    }
}

pub type Dependencies = HashMap<String, Dependency>;

#[derive(Debug)]
pub enum Dependency {
    Simple(String),
    Detailed(DependencyInfo),
}

impl<'de> de::Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct TomlDependencyVisitor;

        impl<'de> de::Visitor<'de> for TomlDependencyVisitor {
            type Value = Dependency;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(
                    "a version string like \"0.9.8\" or a \
                     detailed dependency like { version = \"0.9.8\" }",
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Dependency::Simple(s.to_owned()))
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mvd = de::value::MapAccessDeserializer::new(map);
                DependencyInfo::deserialize(mvd).map(Dependency::Detailed)
            }
        }

        deserializer.deserialize_any(TomlDependencyVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyInfo {
    path: Option<String>,
}

// this impl only used to running tests
impl Default for PkgConfig {
    fn default() -> Self {
        Self { name: Default::default(), version: Version::new(0, 0, 0) }
    }
}

fn default_main_file() -> PathBuf {
    "src/main.l".into()
}

#[derive(Debug, Deserialize)]
pub struct PkgConfig {
    name: String,
    version: Version,
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
