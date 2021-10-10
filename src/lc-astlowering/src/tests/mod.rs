mod validation;

// macro lower_to_ir($src:expr) {
//     lc_driver::Driver::from_src($src).gen_ir().unwrap();
// }

macro expect_lowering_error($src:expr) {
    lc_driver::Driver::from_src($src).gen_ir().unwrap();
}
