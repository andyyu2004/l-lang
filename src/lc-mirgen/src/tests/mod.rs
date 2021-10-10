mod exhaustiveness;

macro analyze($src:expr) {{
    let driver = lc_driver::Driver::from_src($src);
    driver.check().unwrap();
}}

macro expect_analysis_error($src:expr) {{
    let driver = lc_driver::Driver::from_src($src);
    driver.check().unwrap_err();
}}
