use crate::ty::{List, Ty, TyKind};
use crate::typeck::TyCtx;
use smallvec::SmallVec;

crate trait TypeFoldable<'tcx>: Sized {
    /// recursively fold inner `Ty<'tcx>`s
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>;

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>;

    fn fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        self.inner_fold_with(folder)
    }

    fn visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        self.inner_visit_with(visitor)
    }
}

impl<'tcx> TypeFoldable<'tcx> for Ty<'tcx> {
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        let kind = match self.kind {
            TyKind::Array(ty) => TyKind::Array(ty.fold_with(folder)),
            TyKind::Fn(inputs, ret) => TyKind::Fn(inputs.fold_with(folder), ret.fold_with(folder)),
            TyKind::Tuple(tys) => TyKind::Tuple(tys.fold_with(folder)),
            TyKind::Infer(_) | TyKind::Unit | TyKind::Char | TyKind::Num | TyKind::Bool => {
                return self;
            }
        };

        if kind == self.kind { self } else { folder.tcx().mk_ty(kind) }
    }

    fn fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        folder.fold_ty(self)
    }

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        match self.kind {
            TyKind::Fn(inputs, ret) => inputs.visit_with(visitor) || ret.visit_with(visitor),
            TyKind::Tuple(tys) => tys.visit_with(visitor),
            TyKind::Array(ty) => ty.visit_with(visitor),
            TyKind::Infer(_) => false,
            TyKind::Unit | TyKind::Char | TyKind::Num | TyKind::Bool => false,
        }
    }
}

crate trait TypeFolder<'tcx>: Sized {
    fn tcx(&self) -> TyCtx<'tcx>;
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        ty.fold_with(self)
    }
}

crate trait TypeVisitor<'tcx>: Sized {
    fn visit_ty(&mut self, ty: Ty<'tcx>) -> bool;
}

impl<'tcx> TypeFoldable<'tcx> for &'tcx List<Ty<'tcx>> {
    fn inner_fold_with<F: TypeFolder<'tcx>>(&self, folder: &mut F) -> Self {
        fold_list(*self, folder, |tcx, v| tcx.intern_substs(v))
    }

    fn inner_visit_with<V: TypeVisitor<'tcx>>(&self, visitor: &mut V) -> bool {
        self.iter().any(|t| t.visit_with(visitor))
    }
}

impl<'tcx, T> TypeFoldable<'tcx> for &'tcx [T]
where
    T: TypeFoldable<'tcx>,
{
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        folder.tcx().alloc_tir_iter(self.iter().map(|t| t.fold_with(folder)))
    }

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        self.iter().any(|x| x.visit_with(visitor))
    }
}

// from rustc structural_impls.rs
// Does the equivalent of
// ```
// let v = self.iter().map(|p| p.fold_with(folder)).collect::<SmallVec<[_; 8]>>();
// folder.tcx().intern_*(&v)
// ```
/// the same pointer is returned if the list has not changed
fn fold_list<'tcx, F, T>(
    list: &'tcx List<T>,
    folder: &mut F,
    intern: impl FnOnce(TyCtx<'tcx>, &[T]) -> &'tcx List<T>,
) -> &'tcx List<T>
where
    F: TypeFolder<'tcx>,
    T: TypeFoldable<'tcx> + PartialEq + Copy,
{
    let mut iter = list.iter();
    // Look for the first element that changed
    if let Some((i, new_t)) = iter.by_ref().enumerate().find_map(|(i, t)| {
        let new_t = t.fold_with(folder);
        if new_t == t { None } else { Some((i, new_t)) }
    }) {
        // An element changed, prepare to intern the resulting list
        let mut new_list = SmallVec::<[_; 8]>::with_capacity(list.len());
        new_list.extend_from_slice(&list[..i]);
        new_list.push(new_t);
        new_list.extend(iter.map(|t| t.fold_with(folder)));
        intern(folder.tcx(), &new_list)
    } else {
        list
    }
}
