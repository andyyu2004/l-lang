mod def_visitor;
mod late;
mod resolver;
mod scope;

use crate::span::Span;
use def_visitor::DefVisitor;
pub use resolver::{Resolver, ResolverOutputs};
pub use scope::{Scope, Scopes};

#[cfg(test)]
mod tests {
    use crate::exec_expr;

    #[test]
    fn resolve_redeclaration() {
        let _res = exec_expr("let x = 5; let x = x; x").unwrap();
    }
}
