use crate::error::{TypeError, TypeResult};
use crate::ty::*;
use crate::typeck::TyCtx;
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
            (Param(t), Param(u)) if t == u => Ok(a),
            (Infer(_), _) | (_, Infer(_)) => panic!(),
            (Tuple(xs), Tuple(ys)) => self.relate_tuples(xs, ys),
            (Array(t), Array(u)) => self.relate(t, u),
            (&Fn(a, b), &Fn(t, u)) => {
                let s = self.relate(a, t)?;
                let r = self.relate(b, u)?;
                Ok(tcx.mk_ty(TyKind::Fn(s, r)))
            }
            _ => Err(TypeError::Mismatch(a, b)),
        }
    }

    fn relate_tuples(
        &mut self,
        s: SubstRef<'tcx>,
        t: SubstRef<'tcx>,
    ) -> TypeResult<'tcx, Ty<'tcx>> {
        Ok(self.tcx().mk_ty(TyKind::Tuple(self.relate(s, t)?)))
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

impl<'tcx> Relate<'tcx> for SubstRef<'tcx> {
    fn relate(relation: &mut impl TypeRelation<'tcx>, a: Self, b: Self) -> TypeResult<'tcx, Self> {
        if a.len() != b.len() {
            return Err(TypeError::TupleSizeMismatch(a.len(), b.len()));
        }
        let tys: Vec<_> = a.iter().zip(b).map(|(t, u)| relation.relate(t, u)).try_collect()?;
        Ok(relation.tcx().mk_substs(tys.into_iter()))
    }
}
