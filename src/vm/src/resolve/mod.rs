mod def_visitor;
mod late;
mod module;
mod pat;
mod resolver;
mod scope;
#[cfg(test)]
mod tests;

use crate::span::Span;
use def_visitor::DefVisitor;
use late::LateResolver;
use module::{Module, ModuleTree};
use pat::PatternResolutionCtx;
pub use resolver::{Resolutions, Resolver, ResolverArenas};
pub use scope::{Scope, Scopes};

impl<T> std::ops::Index<NS> for PerNS<T> {
    type Output = T;

    fn index(&self, ns: NS) -> &Self::Output {
        match ns {
            NS::Value => &self.value,
            NS::Type => &self.ty,
        }
    }
}

impl<T> std::ops::IndexMut<NS> for PerNS<T> {
    fn index_mut(&mut self, ns: NS) -> &mut Self::Output {
        match ns {
            NS::Value => &mut self.value,
            NS::Type => &mut self.ty,
        }
    }
}
/// namespaces for types and values
#[derive(Debug, Clone, Copy)]
pub enum NS {
    Type,
    Value,
}

/// a `T` for each namespace
#[derive(Default, Debug)]
pub struct PerNS<T> {
    pub value: T,
    pub ty: T,
}
