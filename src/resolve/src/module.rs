use crate::*;
use ast::{Ident, NodeId};
use ir::Res;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::marker::PhantomData;

#[derive(Default, Debug)]
pub struct Module<'a> {
    pub items: RefCell<FxHashMap<Ident, Res<NodeId>>>,
    pub submodules: RefCell<FxHashMap<Ident, ModuleId>>,
    pd: PhantomData<&'a ()>,
}

impl<'a> Module<'a> {
    pub fn root() -> Self {
        let root = Self::default();
        root
    }
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
