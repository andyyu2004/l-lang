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
    let mut ctx = TestCtx::default();
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
        panic!("`{}` errors occured during testing", ctx.errc.get())
    } else {
        eprintln!("passed {} tests in test suite", ctx.testc)
    }
    Ok(())
}

#[derive(Default)]
struct TestCtx {
    errc: Cell<usize>,
    testc: usize,
}

#[derive(Debug)]
struct Output {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl TestCtx {
    pub fn run_recursive(&mut self, path: impl AsRef<Path>, kind: TestKind) -> io::Result<()> {
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

    fn check_test(&mut self, path: &Path, kind: TestKind) -> io::Result<()> {
        match path.extension() {
            Some(ext) if ext.to_str() == Some("l") => self.run_test(&path, kind),
            _ => return Ok(()),
        }
    }

    fn run_compile_fail_test(&mut self, path: &Path) -> io::Result<()> {
        let expected_errors = errors::parse(path);
        let output = self.run(path)?;
        let errc =
            self.with_err_count(|this| this.compare_expected_errors(&expected_errors, &output));
        if errc > 0 {
            eprintln!(
                "compile-fail test at `{}` failed with `{}` error{}",
                path.display(),
                errc,
                util::pluralize!(errc)
            );
        } else {
            eprintln!("compile-fail test at `{}` passed", path.display());
        }
        Ok(())
    }

    /// runs a function `f` and returns how many errors occured during `f`
    fn with_err_count(&mut self, f: impl FnOnce(&mut Self)) -> usize {
        let old_errc = self.errc.get();
        f(self);
        self.errc.get() - old_errc
    }

    fn run(&self, path: &Path) -> io::Result<Output> {
        let mut cmd = Command::new("l");
        cmd.arg("run").arg(path).arg("--error-format=json");
        let output = cmd.output()?;
        let status = output.status;
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        Ok(Output { status, stdout, stderr })
    }

    fn run_test(&mut self, path: &Path, kind: TestKind) -> io::Result<()> {
        self.testc += 1;
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
