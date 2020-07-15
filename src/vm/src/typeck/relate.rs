use super::{TyCtx, TypeFoldable};
use crate::error::{TypeError, TypeResult};
use crate::ty::*;
use itertools::Itertools;

crate trait TypeRelation<'tcx>: Sized {
    fn tcx(&self) -> TyCtx<'tcx>;
    fn relate<T>(&mut self, a: T, b: T) -> TypeResult<'tcx, T>
    where
        T: Relate<'tcx>,
    {
        Relate::relate(self, a, b)
    }

    fn relate_tys(&mut self, a: Ty<'tcx>, b: Ty<'tcx>) -> TypeResult<'tcx, Ty<'tcx>>;

    /// recursively relates the inner types
    /// inference variable cases should be handled before calling this
    /// at this point, we assume `a != b`
    fn relate_inner(&mut self, a: Ty<'tcx>, b: Ty<'tcx>) -> TypeResult<'tcx, Ty<'tcx>> {
        let tcx = self.tcx();
        match (&a.kind, &b.kind) {
            (Infer(_), _) | (_, Infer(_)) => panic!(),
            (Tuple(xs), Tuple(ys)) => self.relate_tuples(xs, ys),
            (Array(t), Array(u)) => self.relate(t, u),
            (Fn(a, b), Fn(t, u)) => todo!(),
            _ => Err(TypeError::Mismatch(a, b)),
        }
    }

    fn relate_tuples(
        &mut self,
        a: SubstRef<'tcx>,
        b: SubstRef<'tcx>,
    ) -> TypeResult<'tcx, Ty<'tcx>> {
        if a.len() != b.len() {
            return Err(TypeError::TupleSizeMismatch(a.len(), b.len()));
        }
        let relations: Vec<_> = a.iter().zip(b).map(|(t, u)| self.relate(t, u)).try_collect()?;
        Ok(self.tcx().mk_tup(relations.into_iter()))
    }
}

crate trait Relate<'tcx>: TypeFoldable<'tcx> + Copy {
    fn relate(relation: &mut impl TypeRelation<'tcx>, a: Self, b: Self) -> TypeResult<'tcx, Self>;
}

impl<'tcx> Relate<'tcx> for Ty<'tcx> {
    fn relate(relation: &mut impl TypeRelation<'tcx>, a: Self, b: Self) -> TypeResult<'tcx, Self> {
        relation.relate_tys(a, b)
    }
}
