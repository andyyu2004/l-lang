#![feature(crate_visibility_modifier)]
#![feature(once_cell)]

mod errors;

use std::cell::Cell;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[derive(Copy, Clone)]
enum TestKind {
    Ui,
    CompileFail,
}

fn main() -> io::Result<()> {
    let ctx = TestCtx::default();
    // assume path is relative to `runner`
    // this will be the case when invoked from a #[test]
    let tests_root = PathBuf::from("../ltests").canonicalize()?;
    dbg!(tests_root.display());
    // this cds to the root `l` directory
    env::set_current_dir("../../")?;
    // install a release build of the compiler locally
    let status = Command::new("cargo").args(&["install", "--path", "src/l"]).status()?;
    assert!(status.success());
    ctx.run_recursive("tests/ltests/compile-fail", TestKind::CompileFail)?;

    if ctx.errc.get() > 0 {
        panic!("errors occured during testing")
    }
    Ok(())
}

#[derive(Default)]
struct TestCtx {
    errc: Cell<usize>,
}

#[derive(Debug)]
struct Output {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl TestCtx {
    pub fn run_recursive(&self, path: impl AsRef<Path>, kind: TestKind) -> io::Result<()> {
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

    crate fn report_error(&self, error: impl Error) {
        self.errc.set(1 + self.errc.get());
        eprintln!("{}", error)
    }

    fn check_test(&self, path: &Path, kind: TestKind) -> io::Result<()> {
        match path.extension() {
            Some(ext) if ext.to_str() == Some("l") => self.run_test(&path, kind),
            _ => return Ok(()),
        }
    }

    fn run_compile_fail_test(&self, path: &Path) -> io::Result<()> {
        let expected_errors = errors::parse(path);
        let output = self.run(path)?;
        assert!(!output.status.success());
        self.compare_expected_errors(&expected_errors, &output);
        Ok(())
    }

    fn run(&self, path: &Path) -> io::Result<Output> {
        let mut cmd = Command::new("l");
        cmd.arg("run").arg(path);
        let output = cmd.output()?;
        let status = output.status;
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        Ok(Output { status, stdout, stderr })
    }

    fn run_test(&self, path: &Path, kind: TestKind) -> io::Result<()> {
        match kind {
            TestKind::Ui => todo!(),
            TestKind::CompileFail => self.run_compile_fail_test(path),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn run_compiler_test_suite() -> std::io::Result<()> {
        crate::main()
    }
}
