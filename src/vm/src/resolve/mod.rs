mod def_visitor;
mod late;
mod module;
mod resolver;
mod scope;
#[cfg(test)]
mod tests;

use crate::span::Span;
use def_visitor::DefVisitor;
use module::{Module, ModuleTree};
pub use resolver::{Resolver, ResolverArenas, ResolverOutputs};
pub use scope::{Scope, Scopes};
