use super::*;

// check Self type exists inside impl scope
#[test]
fn resolve_self_in_impl() {
    let src = r#"
        trait Default {
            fn default() -> Self;
        }"#;

    resolve!(src);
}
