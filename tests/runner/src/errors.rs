use crate::{Output, TestCtx};
use error::{JsonDiagnostic, Severity};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
        .filter_map(|(i, line)| self::parse_line(i + 1, &line.unwrap()))
        .collect()
}

impl TestCtx {
    crate fn compare_expected_errors(&self, expected: &[Error], output: &Output) {
        let errors = serde_json::from_str::<Vec<JsonDiagnostic>>(&output.stderr).unwrap();
        dbg!(expected);
        dbg!(errors);
        todo!()
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
