use crate::NewCmd;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};


crate fn new(config: NewCmd) -> io::Result<()> {
    let path = &config.path;
    let name = path.file_name().unwrap().to_str().unwrap();
    fs::create_dir_all(path)?;
    env::set_current_dir(path)?;

    let mut file = File::create("L.toml")?;
    file.write(
        format!(
            r#"[package]
name = "{}"
version = "0.1.0"

[dependencies]
 "#,
            name
        )
        .as_bytes(),
    )?;

    fs::create_dir("src")?;
    env::set_current_dir("src")?;
    let mut file = File::create("main.l")?;
    file.write(r#"fn main() -> int { 0 }"#.as_bytes())?;

    Ok(())
}
