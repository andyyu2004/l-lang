/// note how f is called in main before it is "defined"
fn hoisted_function() {
    let src = r#"
        fn main() -> number {
            f()
        }

        fn f() -> number {
            1
        }
    "#;
    compile!(src);
}
