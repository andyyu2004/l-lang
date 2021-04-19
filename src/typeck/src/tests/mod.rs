mod closure_tests;
mod collection_tests;
mod deref_tests;
mod enum_tests;
mod fn_tests;
mod general_tests;
mod parametric_tests;
mod pattern_tests;
mod struct_tests;
mod generics_tests;

use itertools::Itertools;

fn wrap_in_main(src: &str) -> String {
    format!("fn main() -> int {{ {} }}", src)
}

macro expect_type_error($src:expr) {{
    let driver = ldriver::Driver::from_src($src);
    driver.check().unwrap_err();
}}

macro expect_type_error_expr($src:expr) {{ expect_type_error!(&wrap_in_main($src)) }}

macro typeck($src:expr) {{
    let driver = ldriver::Driver::from_src($src);
    driver.check().unwrap();
}}

macro tir($src:expr) {{
    let driver = ldriver::Driver::from_src($src);
    let tir = driver.gen_tir().unwrap();
    tir.to_string()
}}

macro typeck_expr($src:expr) {{ tir!(&wrap_in_main($src)) }}

macro lines($s:expr) {{
    let mut lines = $s.lines();
    lines.next().unwrap();
    lines.next().unwrap();
    lines.next_back().unwrap();
    lines.map(|line| line.trim()).collect_vec()
}}
