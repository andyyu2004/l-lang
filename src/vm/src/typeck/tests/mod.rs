mod fn_tests;
mod lambda_tests;

use crate::driver::Driver;
use crate::wrap_in_main;
use itertools::Itertools;

macro typeck_prog($src:expr) {{
    let driver = Driver::new($src);
    let tir = driver.gen_tir().unwrap();
    tir.to_string()
}}

macro typeck($src:expr) {{ typeck_prog!(&wrap_in_main($src)) }}

macro lines($s:expr) {{
    let mut lines = $s.lines();
    lines.next().unwrap();
    lines.next_back().unwrap();
    lines.map(|line| line.trim()).collect_vec()
}}

// only works for single line expressions
fn remove_surrounding_block(s: &str) -> &str {
    s.lines().nth(1).unwrap().trim()
}
