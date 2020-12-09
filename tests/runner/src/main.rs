use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Copy, Clone)]
enum TestKind {
    Ui,
}

fn main() -> io::Result<()> {
    let ctx = TestCtx {};
    // path is relative to `runner`
    let tests_root = PathBuf::from("../ltests").canonicalize()?;
    dbg!(tests_root.display());
    let handle = Command::new("l").arg("../ltests").spawn()?;
    let output = handle.wait_with_output()?;
    assert!(output.status.success());
    // ctx.run_recursive(&Path::new(""))?;
    Ok(())
}

struct TestCtx {}

impl TestCtx {
    pub fn run_recursive(&self, path: &impl AsRef<Path>, kind: TestKind) -> io::Result<()> {
        let dir = fs::read_dir(path)?;
        for entry in dir {
            let entry = entry?;
            if entry.metadata()?.is_dir() {
                self.run_recursive(&entry.path(), kind)?;
            } else {
                self.check_test(&entry.path(), kind)?;
            }
        }
        Ok(())
    }

    fn check_test(&self, path: &Path, kind: TestKind) -> io::Result<()> {
        match path.extension() {
            Some(ext) if ext.to_str() == Some("l") => self.run_test(&path, kind),
            _ => return Ok(()),
        }
    }

    fn run_test(&self, path: &Path, kind: TestKind) -> io::Result<()> {
        // let dir_path = path.parent().unwrap();
        // let driver = ldriver::Driver::new(&dir_path);
        // Command::new("cargo rj");
        // dbg!(driver.llvm_exec()).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn run_compiler_test_suite() -> std::io::Result<()> {
        crate::main()
    }
}
