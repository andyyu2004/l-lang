use crate::ast::Ident;
use rustc_hash::FxHashMap;

#[derive(Debug, Deref, DerefMut)]
pub struct Scopes<T> {
    scopes: Vec<Scope<T>>,
}

impl<T> Default for Scopes<T> {
    fn default() -> Self {
        Self { scopes: Default::default() }
    }
}

impl<T> Scopes<T> {
    /// returns index for next param
    pub fn new_ty_param(&mut self) -> usize {
        let index = self.paramc();
        self.curr_scope_mut().paramc += 1;
        index
    }

    fn paramc(&self) -> usize {
        self.scopes.iter().map(|s| s.paramc).sum::<usize>()
    }

    fn curr_scope_mut(&mut self) -> &mut Scope<T> {
        self.scopes.last_mut().expect("ran out of scopes")
    }

    pub fn def(&mut self, ident: Ident, value: T) {
        self.curr_scope_mut().def(ident, value);
    }

    pub fn lookup(&self, ident: &Ident) -> Option<&T> {
        for scope in self.scopes.iter().rev() {
            if let Some(x) = scope.lookup(ident) {
                return Some(x);
            }
        }
        None
    }
}

impl<T> Default for Scope<T> {
    fn default() -> Self {
        Self { bindings: Default::default(), paramc: Default::default() }
    }
}

#[derive(Debug)]
pub struct Scope<T> {
    bindings: FxHashMap<Ident, T>,
    paramc: usize,
}

impl<T> Scope<T> {
    fn def(&mut self, ident: Ident, value: T) -> Option<T> {
        self.bindings.insert(ident, value)
    }

    fn lookup(&self, ident: &Ident) -> Option<&T> {
        self.bindings.get(ident)
    }
}
