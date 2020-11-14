use crate::*;
use ast::{Ident, NodeId};
use ir::Res;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::marker::PhantomData;

#[derive(Default, Debug)]
pub struct Mod<'a> {
    pub items: RefCell<FxHashMap<Ident, Res<NodeId>>>,
    pub submodules: RefCell<FxHashMap<Ident, ModuleId>>,
    pd: PhantomData<&'a ()>,
}
