use crate::ty::{FnSig, List, Ty, TyCtx, TyKind};
use crate::ArenaAllocatable;
use smallvec::SmallVec;

pub trait TypeFoldable<'tcx>: Sized {
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

impl<'tcx> TypeFoldable<'tcx> for FnSig<'tcx> {
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        Self { params: self.params.fold_with(folder), ret: self.ret.fold_with(folder) }
    }

    fn inner_visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        self.params.visit_with(visitor) || self.ret.visit_with(visitor)
    }
}

impl<'tcx> TypeFoldable<'tcx> for Ty<'tcx> {
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        let kind = match self.kind {
            TyKind::FnPtr(fn_ty) => TyKind::FnPtr(fn_ty.fold_with(folder)),
            TyKind::FnDef(def_id, substs) => TyKind::FnDef(def_id, substs.fold_with(folder)),
            TyKind::Box(ty) => TyKind::Box(ty.fold_with(folder)),
            TyKind::Ptr(ty) => TyKind::Ptr(ty.fold_with(folder)),
            TyKind::Array(ty, n) => TyKind::Array(ty.fold_with(folder), n),
            TyKind::Tuple(tys) => TyKind::Tuple(tys.fold_with(folder)),
            TyKind::Adt(adt, substs) => TyKind::Adt(adt, substs.fold_with(folder)),
            TyKind::Opaque(def, substs) => TyKind::Opaque(def, substs.fold_with(folder)),
            TyKind::Param(_)
            | TyKind::Infer(_)
            | TyKind::Char
            | TyKind::Discr
            | TyKind::Never
            | TyKind::Int
            | TyKind::Bool
            | TyKind::Float
            | TyKind::Error => {
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
            TyKind::FnPtr(sig) => sig.visit_with(visitor),
            TyKind::Ptr(ty) | TyKind::Box(ty) | TyKind::Array(ty, _) => ty.visit_with(visitor),
            TyKind::Tuple(tys) => tys.visit_with(visitor),
            TyKind::Opaque(_, substs) => substs.visit_with(visitor),
            TyKind::FnDef(_, substs) | TyKind::Adt(_, substs) => substs.visit_with(visitor),
            TyKind::Param(..)
            | TyKind::Infer(..)
            | TyKind::Discr
            | TyKind::Never
            | TyKind::Error
            | TyKind::Char
            | TyKind::Int
            | TyKind::Float
            | TyKind::Bool => false,
        }
    }

    fn visit_with<V>(&self, visitor: &mut V) -> bool
    where
        V: TypeVisitor<'tcx>,
    {
        visitor.visit_ty(self)
    }
}

pub trait TypeFolder<'tcx>: Sized {
    fn tcx(&self) -> TyCtx<'tcx>;
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        ty.fold_with(self)
    }
}

pub trait TypeVisitor<'tcx>: Sized {
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
    T: TypeFoldable<'tcx> + ArenaAllocatable<'tcx>,
{
    fn inner_fold_with<F>(&self, folder: &mut F) -> Self
    where
        F: TypeFolder<'tcx>,
    {
        folder.tcx().alloc_iter(self.iter().map(|t| t.fold_with(folder)))
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
