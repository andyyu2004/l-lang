#![feature(crate_visibility_modifier)]
#![feature(once_cell)]

mod errors;

use lc_error::ErrorFormat;
use std::cell::Cell;
use std::env;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

#[derive(Copy, Clone)]
enum TestKind {
    /// tests the output of of the compiler
    /// captures and diffs stdout and stderr
    Ui,
    /// less precise but easier to write ui tests
    CompileFail,
    /// runs the program and diffs the output of the program (stdout)
    Output,
}

impl Display for TestKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            TestKind::Ui => "ui",
            TestKind::CompileFail => "compile-fail",
            TestKind::Output => "output",
        };
        write!(f, "{}", s)
    }
}

fn main() -> io::Result<()> {
    // assume path is relative to root of `runner` crate
    // this will be the case when invoked from #[test]
    // let tests_root = PathBuf::from("../ltests").canonicalize()?;
    // this cds to the root `l` directory
    env::set_current_dir("../../")?;
    // install a release build of the compiler locally
    let status = Command::new("cargo").args(&["install", "--path", "src/l"]).status()?;
    assert!(status.success());

    TestCtx::default().run_test_suite()?;

    Ok(())
}

#[derive(Default)]
struct TestCtx {
    errc: Cell<usize>,
    testc: usize,
}

#[derive(Debug)]
struct Output {
    stdout: String,
    stderr: String,
}

impl TestCtx {
    fn run_test_suite(&mut self) -> io::Result<()> {
        self.run_recursive("tests/ltests/compile-fail", TestKind::CompileFail)?;
        self.run_recursive("tests/ltests/ui", TestKind::Ui)?;
        self.run_recursive("tests/ltests/output", TestKind::Output)?;

        let errc = self.errc.get();
        if errc > 0 {
            panic!("{} error{} occured", errc, lc_util::pluralize!(errc))
        } else {
            eprintln!("passed {} tests", self.testc)
        }
        Ok(())
    }

    fn run_recursive(&mut self, path: impl AsRef<Path>, kind: TestKind) -> io::Result<()> {
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

    pub(crate) fn inc_errc(&self) {
        self.errc.set(1 + self.errc.get());
    }

    pub(crate) fn report_error(&self, error: impl Error) {
        self.inc_errc();
        eprintln!("{}", error)
    }

    fn check_test(&mut self, path: &Path, kind: TestKind) -> io::Result<()> {
        match path.extension() {
            Some(ext) if ext.to_str() == Some("l") => {
                let errc = self.with_err_count(|this| this.run_test(&path, kind))?;
                if errc > 0 {
                    eprintln!(
                        "failed {} test at `{}` with `{}` error{}",
                        kind,
                        path.display(),
                        errc,
                        lc_util::pluralize!(errc)
                    );
                } else {
                    eprintln!("passed {} test at `{}`", kind, path.display());
                }
                Ok(())
            }
            _ => return Ok(()),
        }
    }

    fn run_output_test(&mut self, path: &Path) -> io::Result<()> {
        let output = self.run(path, ErrorFormat::Text)?;
        let mut stdout_path = path.to_path_buf();
        assert!(stdout_path.set_extension("stdout"));
        let expected_stdout = fs::read_to_string(stdout_path).ok().unwrap();
        self.compare(&output.stdout, &expected_stdout);
        Ok(())
    }

    fn run_ui_test(&mut self, path: &Path) -> io::Result<()> {
        let output = self.run(path, ErrorFormat::Text)?;
        let mut stdout_path = path.to_path_buf();
        assert!(stdout_path.set_extension("stdout"));
        let mut stderr_path = path.to_path_buf();
        assert!(stderr_path.set_extension("stderr"));

        // if no file, it means there should be no output
        let expected_stdout = fs::read_to_string(stdout_path).ok().unwrap_or_else(String::new);
        let expected_stderr = fs::read_to_string(stderr_path).ok().unwrap_or_else(String::new);

        // remove ansi escape codes
        let processed_stderr =
            String::from_utf8(strip_ansi_escapes::strip(&output.stderr).unwrap()).unwrap();

        self.compare(&output.stdout, &expected_stdout);
        self.compare(&processed_stderr, &expected_stderr);
        Ok(())
    }

    fn run_compile_fail_test(&mut self, path: &Path) -> io::Result<()> {
        let output = self.run(path, ErrorFormat::Json)?;
        let expected_errors = errors::parse(path);
        self.compare_expected_errors(&expected_errors, &output);
        Ok(())
    }

    /// runs a function `f` and returns how many errors occured during `f`
    fn with_err_count(&mut self, f: impl FnOnce(&mut Self) -> io::Result<()>) -> io::Result<usize> {
        let old_errc = self.errc.get();
        f(self)?;
        Ok(self.errc.get() - old_errc)
    }

    fn run(&self, path: &Path, error_format: ErrorFormat) -> io::Result<Output> {
        let mut cmd = Command::new("l");
        cmd.arg("run").arg(path);
        cmd.arg(format!("--error-format={}", error_format));
        let output = cmd.output()?;
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        Ok(Output { stdout, stderr })
    }

    fn run_test(&mut self, path: &Path, kind: TestKind) -> io::Result<()> {
        self.testc += 1;
        match kind {
            TestKind::Ui => self.run_ui_test(path),
            TestKind::CompileFail => self.run_compile_fail_test(path),
            TestKind::Output => self.run_output_test(path),
        }
    }

    fn compare<T: PartialEq + Debug>(&mut self, x: T, y: T) {
        if x == y {
            return;
        }
        self.inc_errc();
        eprintln!("{:?} != {:?}", x, y);
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn run_compiler_test_suite() -> std::io::Result<()> {
        crate::main()
    }
}
