mod fn_tests;
mod general_tests;
mod lambda_tests;
mod struct_tests;

use crate::driver::Driver;
use crate::wrap_in_main;
use itertools::Itertools;

macro typeck($src:expr) {{
    let driver = Driver::new($src);
    let tir = driver.gen_mir().unwrap();
    tir.to_string()
}}

macro typeck_expr($src:expr) {{ typeck!(&wrap_in_main($src)) }}

macro lines($s:expr) {{
    let mut lines = $s.lines();
    lines.next().unwrap();
    lines.next().unwrap();
    lines.next_back().unwrap();
    lines.map(|line| line.trim()).collect_vec()
}}
