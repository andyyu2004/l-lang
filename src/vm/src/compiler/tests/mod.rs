mod local_variables;
use crate::driver::Driver;

pub macro compile($src:expr) {{
    let driver = Driver::new($src);
    driver.compile().unwrap()
}}
