use crate::CodegenCtx;
use ir::{DefId, FnVisitor, ItemVisitor};
use lcore::mir::Operand;
use lcore::ty::{Instance, InstanceKind, Subst, TyCtx, TypeFoldable};
use mir::{TyCtxMirExt, Visitor};
use rustc_hash::FxHashSet;
use std::ops::Deref;

pub trait Monomorphize<'tcx> {
    fn monomorphize<T>(&self, t: T) -> T
    where
        T: TypeFoldable<'tcx>;
}

impl<'tcx> CodegenCtx<'tcx> {
    /// collects all references to generic items along with substitutions representing
    /// each unique instantiation of the generic parameters
    pub fn collect_monomorphization_instances(&self) -> FxHashSet<Instance<'tcx>> {
        let roots = RootCollector::new(self.tcx).collect_roots();
        let mut instances = FxHashSet::default();
        for root in roots {
            let instance = Instance::mono_item(root);
            self.collect_rec(&mut instances, instance);
        }
        instances
    }

    fn collect_rec(&self, visited: &mut FxHashSet<Instance<'tcx>>, instance: Instance<'tcx>) {
        visited.insert(instance);
        assert!(!self.instance_mir.borrow().contains_key(&instance));
        // TODO this will generate the same mir multiple times
        let mir = match instance.kind {
            InstanceKind::Item => self.tcx.mir_of_def(instance.def_id),
        };

        if let Ok(mir) = mir {
            println!("{}", mir);
            self.instance_mir.borrow_mut().insert(instance, mir);
            MonoCollector { cctx: self, instance, visited }.visit_mir(mir)
        }
    }
}

/// collects all the non-generic `roots`
struct RootCollector<'tcx> {
    tcx: TyCtx<'tcx>,
    roots: Vec<DefId>,
}

impl<'tcx> RootCollector<'tcx> {
    pub fn collect_roots(mut self) -> Vec<DefId> {
        self.visit_ir(self.tcx.ir);
        self.roots
    }

    pub fn new(tcx: TyCtx<'tcx>) -> Self {
        Self { tcx, roots: Default::default() }
    }
}

impl<'tcx> FnVisitor<'tcx> for RootCollector<'tcx> {
    fn visit_fn(
        &mut self,
        def_id: ir::DefId,
        _ident: ast::Ident,
        _sig: &'tcx ir::FnSig<'tcx>,
        generics: &'tcx ir::Generics<'tcx>,
        _body: &'tcx ir::Body<'tcx>,
    ) {
        if !generics.params.is_empty() {
            return;
        }
        self.roots.push(def_id)
    }
}

/// finds all instantiations of generic items starting from a given (non-polymorphic) root
struct MonoCollector<'a, 'tcx> {
    cctx: &'a CodegenCtx<'tcx>,
    instance: Instance<'tcx>,
    visited: &'a mut FxHashSet<Instance<'tcx>>,
}

impl<'a, 'tcx> Monomorphize<'tcx> for MonoCollector<'a, 'tcx> {
    fn monomorphize<T>(&self, t: T) -> T
    where
        T: TypeFoldable<'tcx>,
    {
        t.subst(self.cctx.tcx, self.instance.substs)
    }
}

impl<'a, 'tcx> Visitor<'tcx> for MonoCollector<'a, 'tcx> {
    fn visit_operand(&mut self, operand: &Operand<'tcx>) {
        // `Operand::Item` is the only way to reference a generic item
        if let &Operand::Item(def_id, fn_ty) = operand {
            // firstly, we must monomorphize the ty with its
            // "parent" instance's substitutions
            let mono_ty = self.monomorphize(fn_ty);
            // `ty` should have no type parameters after monomorphization
            assert!(!mono_ty.has_ty_params());
            // this `substs` is the substitution
            // applied to the generic function with def_id `def_id`
            // to obtain its concrete type
            let scheme = self.collected_ty(def_id);
            let substs = self.match_tys(scheme, mono_ty);
            let instance = Instance::item(substs, def_id);
            if let Some(prev) =
                self.cctx.operand_instance_map.borrow_mut().insert((def_id, mono_ty), instance)
            {
                // the same operand key shouldn't map to different instances
                assert_eq!(prev, instance);
            }

            if self.visited.contains(&instance) {
                return;
            }
            // recursively collect all its neighbours
            self.cctx.collect_rec(self.visited, instance);
        }
    }
}

impl<'a, 'tcx> Deref for MonoCollector<'a, 'tcx> {
    type Target = TyCtx<'tcx>;

    fn deref(&self) -> &Self::Target {
        &self.cctx.tcx
    }
}
