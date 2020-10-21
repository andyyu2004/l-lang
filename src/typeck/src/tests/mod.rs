mod closure_tests;
mod collection_tests;
mod deref_tests;
mod enum_tests;
mod fn_tests;
mod general_tests;
mod parametric_tests;
mod struct_tests;

use itertools::Itertools;

fn wrap_in_main(src: &str) -> String {
    format!("fn main() -> int {{ {} }}", src)
}

macro expect_error($src:expr) {{
    let driver = ldriver::Driver::new($src);
    driver.gen_tir().unwrap_err();
}}

macro expect_error_expr($src:expr) {{ expect_error!(&wrap_in_main($src)) }}

macro typeck($src:expr) {{
    let driver = ldriver::Driver::new($src);
    let tir = driver.gen_tir().unwrap();
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
