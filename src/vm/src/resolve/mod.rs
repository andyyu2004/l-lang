mod def_visitor;
mod late;
mod module;
mod resolver;
mod scope;

use crate::span::Span;
use def_visitor::DefVisitor;
use module::{Module, ModuleTree};
pub use resolver::{Resolver, ResolverArenas, ResolverOutputs};
pub use scope::{Scope, Scopes};

#[cfg(test)]
mod tests {
    use crate::llvm_exec_expr;

    #[test]
    fn resolve_redeclaration() {
        let _res = llvm_exec_expr("let x = 5; let x = x; x").unwrap();
    }
}
