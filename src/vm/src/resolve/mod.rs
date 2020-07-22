mod item;
mod late;
mod resolver;
mod scope;

use crate::span::Span;
crate use resolver::{Resolver, ResolverOutputs};
crate use scope::{Scope, Scopes};

#[cfg(test)]
mod tests {
    use crate::exec_expr;

    #[test]
    fn resolve_redeclaration() {
        let _res = exec_expr("let x = 5; let x = x; x").unwrap();
    }
}
