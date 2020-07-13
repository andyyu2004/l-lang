use super::{TyCtx, TypeFoldable};
use crate::{error::TypeResult, ty::Ty};

crate trait TypeRelation<'tcx>: Sized {
    fn tcx(&self) -> TyCtx<'tcx>;
    fn relate<T>(&mut self, a: T, b: T) -> TypeResult<'tcx, T>
    where
        T: Relate<'tcx>,
    {
        Relate::relate(self, a, b)
    }

    fn relate_tys(&mut self, a: Ty<'tcx>, b: Ty<'tcx>) -> TypeResult<'tcx, Ty<'tcx>>;
}

crate trait Relate<'tcx>: TypeFoldable<'tcx> + Copy {
    fn relate(relation: &mut impl TypeRelation<'tcx>, a: Self, b: Self) -> TypeResult<'tcx, Self>;
}

impl<'tcx> Relate<'tcx> for Ty<'tcx> {
    fn relate(relation: &mut impl TypeRelation<'tcx>, a: Self, b: Self) -> TypeResult<'tcx, Self> {
        relation.relate_tys(a, b)
    }
}
