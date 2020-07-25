mod conditionals;
mod local_variables;

use crate::driver::Driver;
use crate::wrap_in_main;

pub macro compile($src:expr) {{
    let driver = Driver::new($src);
    driver.compile().unwrap()
}}

pub macro compile_expr($src:expr) {{
    let src = wrap_in_main($src);
    compile!(&src)
}}
