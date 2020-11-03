mod validation;

macro lower_to_ir($src:expr) {
    ldriver::Driver::from_src($src).gen_ir().unwrap();
}

macro expect_error($src:expr) {
    ldriver::Driver::from_src($src).gen_ir().unwrap();
}
