use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

fn main() -> io::Result<()> {
    let ctx = TestCtx {};
    ctx.run_recursive(&Path::new(""))?;
    Ok(())
}

enum TestKind {
    Ui,
}

struct TestCtx {}

impl TestCtx {
    pub fn run_recursive(&self, path: &impl AsRef<Path>) -> io::Result<()> {
        let dir = fs::read_dir(path)?;
        for entry in dir {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                self.run_recursive(&entry.path())?;
            } else {
                self.check_test(&entry.path())?;
            }
        }
        Ok(())
    }

    fn check_test(&self, path: &Path) -> io::Result<()> {
        match path.extension() {
            Some(ext) if ext.to_str() == Some("l") => self.run_test(&path),
            _ => return Ok(()),
        }
    }

    fn run_test(&self, path: &Path) -> io::Result<()> {
        let dir_path = path.parent().unwrap();
        let src = fs::read_to_string(path)?;
        let driver = ldriver::Driver::new(&src);
        // Command::new("cargo rj");
        dbg!(driver.llvm_exec()).unwrap();
        Ok(())
    }
}
