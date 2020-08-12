//! escape analysis of upvars
//! (to decide whether they can safely live on the stack or if we must allocate on the heap)

use crate::ir;
use rustc_hash::FxHashMap;
use std::marker::PhantomData;

#[derive(Default, Debug)]
struct Scopes {
    scopes: Vec<FxHashMap<ir::Id, ()>>,
}

impl Scopes {
    pub fn def(&mut self, id: ir::Id) {
        self.scopes.last_mut().unwrap().insert(id, ());
    }

    pub fn lookup(&mut self, id: ir::Id) {
        for scope in self.scopes.iter().rev() {
            match scope.get(&id) {
                Some(&x) => x,
                None => continue,
            }
        }
    }
}

#[derive(Debug)]
pub struct UpvarId {
    /// `Id` of the variable that this `Upvar` captures
    id: ir::Id,
    /// `Id` of the body of the lambda that closes over the `Upvar`
    lambda: ir::Id,
}

#[derive(Default, Debug)]
pub struct EscapeVisitor<'ir> {
    _marker: PhantomData<&'ir ()>,
    /// these scopes don't represent lexical scopes as in the resolver.
    /// instead, they represent function scopes, as functions are the only places where
    ///
    scopes: Scopes,
    upvars: FxHashMap<UpvarId, ()>,
}

impl<'ir> EscapeVisitor<'ir> {
    fn with_lambda_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let ret = f(self);
        ret
    }

    fn def_pattern(&mut self, pat: &'ir ir::Pattern<'ir>) {
        match pat.kind {
            ir::PatternKind::Binding(ident, _) => {}
            ir::PatternKind::Wildcard => {}
            ir::PatternKind::Tuple(_) => {}
            ir::PatternKind::Lit(_) => panic!(),
        }
    }
}

impl<'ir> ir::Visitor<'ir> for EscapeVisitor<'ir> {
    fn visit_lambda(&mut self, sig: &'ir ir::FnSig<'ir>, body: &'ir ir::Body) {
        self.with_lambda_scope(|this| ir::walk_body(this, body))
    }

    fn visit_pat(&mut self, pat: &'ir ir::Pattern<'ir>) {
        self.def_pattern(pat);
        ir::walk_pat(self, pat);
    }
}
