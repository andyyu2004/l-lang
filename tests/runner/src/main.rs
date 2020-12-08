use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
        // let dir_path = path.parent().unwrap();
        // let driver = ldriver::Driver::new(&dir_path);
        // Command::new("cargo rj");
        // dbg!(driver.llvm_exec()).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use std::path::PathBuf;
    use std::process::Command;

    #[test]
    fn run_compiler_test_suite() -> io::Result<()> {
        // path is relative to `runner`
        let tests_root = PathBuf::from("../ltests").canonicalize()?;
        dbg!(tests_root.display());
        let handle = Command::new("l").arg("../ltests").spawn()?;
        let output = handle.wait_with_output()?;
        assert!(output.status.success());
        Ok(())
    }
}
