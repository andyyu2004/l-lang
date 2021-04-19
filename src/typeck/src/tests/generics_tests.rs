use super::*;

/// non cargo test version of `incorrect_number_of_generic_args_trait_impl.l`
#[test]
fn incorrect_number_of_generic_parameters_trait_impl() {
    let src = r#"
    enum Enum<T> {}
    trait SomeTrait {}


    impl SomeTrait for Enum {
    }"#;
    expect_type_error!(src);
}
