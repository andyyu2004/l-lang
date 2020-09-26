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
pub use resolver::{Resolutions, Resolver, ResolverArenas};
pub use scope::{Scope, Scopes};
