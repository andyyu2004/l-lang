mod exhaustiveness;

macro analyze($src:expr) {{
    let driver = ldriver::Driver::from_src($src);
    driver.check().unwrap();
}}

macro expect_analysis_error($src:expr) {{
    let driver = ldriver::Driver::from_src($src);
    driver.check().unwrap_err();
}}
