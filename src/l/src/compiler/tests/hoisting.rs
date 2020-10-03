/// note how `f` is called in `main` before it is "defined"
fn hoisted_function() {
    let src = r#"
        fn main() -> int {
            f()
        }

        fn f() -> int {
            1
        }
    "#;
    compile!(src);
}
