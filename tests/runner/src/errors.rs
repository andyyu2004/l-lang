use crate::{Output, TestCtx};
use error::{JsonDiagnostic, Severity};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TestFailure {
    #[error("expected `{0}` errors, but found `{1}`")]
    UnexpectedNumberOfErrors(usize, usize),
}

#[derive(Debug)]
pub struct Error {
    line_number: usize,
    severity: Severity,
    msg: String,
}

crate fn parse(path: impl AsRef<Path>) -> Vec<Error> {
    let reader = BufReader::new(File::open(path).unwrap());
    reader
        .lines()
        .enumerate()
        // +1 as line count starts from one in error messages
        .filter_map(|(i, line)| self::parse_line(1 + i, &line.unwrap()))
        .collect()
}

macro_rules! test_assert_eq {
    ($left:expr, $right:expr $(,)?) => ({
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#, &*left_val, &*right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`: {}"#, &*left_val, &*right_val,
                           $crate::format_args!($($arg)+))
                }
            }
        }
    });
}

impl TestCtx {
    crate fn compare_expected_errors(&self, expected: &[Error], output: &Output) {
        let mut errors = serde_json::from_str::<Vec<JsonDiagnostic>>(&output.stderr).unwrap();
        if errors.len() != expected.len() {
            return self
                .report_error(TestFailure::UnexpectedNumberOfErrors(expected.len(), errors.len()));
        }

        errors.sort_unstable_by_key(|err| err.line);

        for (actual, expected) in errors.iter().zip(expected) {
            test_assert_eq!(actual.line, expected.line_number);
            test_assert_eq!(actual.severity, expected.severity);
            // we just compare the first line of the actual message in this test
            test_assert_eq!(actual.msg.lines().next().unwrap(), expected.msg);
        }
    }
}

/**
 * `//~`
 * `//~^^` means expect warning 2 lines above
 * */
fn parse_line(line_number: usize, line: &str) -> Option<Error> {
    let idx = 3 + line.find("//~")?;
    let line = &line[idx..];
    let above = line.chars().take_while(|&c| c == '^').count();
    let line_number = line_number - above;
    let line = &line[above..];

    let line = line.trim_start();
    let next_whitespace = line.find(' ')?;
    let kind = self::parse_severity(&line[..next_whitespace]);
    let msg = line[next_whitespace..].trim().to_owned();
    Some(Error { line_number, severity: kind, msg })
}

fn parse_severity(line: &str) -> Severity {
    match line.trim_start() {
        "ERROR" => Severity::Error,
        "WARNING" => Severity::Warning,
        _ => panic!("invalid error kind `{}`", line),
    }
}
