mod uninit_tests;

macro expect_analysis_error($src:expr) {{
    let driver = lc_driver::Driver::from_src($src);
    // we don't really have a driver step called `gen_mir`
    // so we run the next closest thing which also compiles it
    // to llvm ir, but runs mirgen and analysis as an intermediate step
    driver.llvm_compile().unwrap_or_else(|_| panic!("expected mirgen/mir analysis error"));
}}

macro analyze($src:expr) {{
    let driver = lc_driver::Driver::from_src($src);
    driver.llvm_compile().unwrap();
}}
