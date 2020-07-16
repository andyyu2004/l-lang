use crate::ast::Ident;
use rustc_hash::FxHashMap;

#[derive(Debug, Deref, DerefMut)]
crate struct Scopes<T> {
    scopes: Vec<Scope<T>>,
}

impl<T> Default for Scopes<T> {
    fn default() -> Self {
        Self { scopes: Default::default() }
    }
}

impl<T> Default for Scope<T> {
    fn default() -> Self {
        Self { bindings: Default::default() }
    }
}

#[derive(Debug)]
crate struct Scope<T> {
    bindings: FxHashMap<Ident, T>,
}

impl<T> Scopes<T> {
    pub fn def(&mut self, ident: Ident, value: T) {
        self.scopes.last_mut().unwrap().def(ident, value);
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

impl<T> Scope<T> {
    pub fn def(&mut self, ident: Ident, value: T) -> Option<T> {
        self.bindings.insert(ident, value)
    }

    pub fn lookup(&self, ident: &Ident) -> Option<&T> {
        self.bindings.get(ident)
    }
}
