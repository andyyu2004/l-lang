
use crate::resolve;

// check Self type exists inside impl scope
#[test]
fn resolve_self_in_impl() {
    resolve!({
        trait Default {
            fn default() -> Self;
        }
    });
}
