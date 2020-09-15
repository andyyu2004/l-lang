use crate::ast::{Ident, NodeId};
use crate::ir::{ModuleId, Res, ROOT_MODULE};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::marker::PhantomData;

#[derive(Default, Debug)]
pub struct Module<'a> {
    pub items: RefCell<FxHashMap<Ident, Res<NodeId>>>,
    pub submodules: RefCell<FxHashMap<Ident, ModuleId>>,
    pd: PhantomData<&'a ()>,
}

pub enum ModuleTree<'a> {
    Module(ModuleId),
    Tree(&'a ModuleTree<'a>),
}

impl<'a> Default for ModuleTree<'a> {
    fn default() -> Self {
        Self::Module(ROOT_MODULE)
    }
}
