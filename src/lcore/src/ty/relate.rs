use crate::ty::{self, *};
use itertools::Itertools;

pub trait TypeRelation<'tcx>: Sized {
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
        match (a.kind, b.kind) {
            (ty::Box(t), ty::Box(u)) => self.relate(t, u),
            (ty::Param(t), ty::Param(u)) if t.idx == u.idx => Ok(a),
            (ty::Tuple(xs), ty::Tuple(ys)) => self.relate_tuples(xs, ys),
            (ty::Array(t, m), ty::Array(u, n)) if m == n => self.relate(t, u),
            (ty::Adt(adtx, substsx), ty::Adt(adty, substsy)) if adtx == adty => {
                let substs = self.relate(substsx, substsy)?;
                Ok(tcx.mk_adt_ty(adtx, substs))
            }
            (_, ty::Never) => Ok(a),
            (ty::Never, _) => Ok(b),
            (ty::FnPtr(f), ty::FnPtr(g)) => Ok(tcx.mk_fn_ptr(self.relate(f, g)?)),
            (ty::Infer(_), _) | (_, ty::Infer(_)) => panic!(),
            _ => TypeResult::Err(TypeError::Mismatch(a, b)),
        }
    }

    fn relate_tuples(
        &mut self,
        s: SubstsRef<'tcx>,
        t: SubstsRef<'tcx>,
    ) -> TypeResult<'tcx, Ty<'tcx>> {
        Ok(self.tcx().mk_tup(self.relate(s, t)?))
    }
}

pub trait Relate<'tcx>: TypeFoldable<'tcx> + Copy {
    fn relate(relation: &mut impl TypeRelation<'tcx>, a: Self, b: Self) -> TypeResult<'tcx, Self>;
}

impl<'tcx> Relate<'tcx> for Ty<'tcx> {
    fn relate(relation: &mut impl TypeRelation<'tcx>, a: Self, b: Self) -> TypeResult<'tcx, Self> {
        relation.relate_tys(a, b)
    }
}

impl<'tcx> Relate<'tcx> for SubstsRef<'tcx> {
    fn relate(relation: &mut impl TypeRelation<'tcx>, a: Self, b: Self) -> TypeResult<'tcx, Self> {
        if a.len() != b.len() {
            return TypeResult::Err(TypeError::TupleSizeMismatch(a.len(), b.len()));
        }
        let tys: Vec<_> = a.iter().zip(b).map(|(t, u)| relation.relate(t, u)).try_collect()?;
        Ok(relation.tcx().mk_substs(tys))
    }
}
impl<'tcx> Relate<'tcx> for FnSig<'tcx> {
    fn relate(relation: &mut impl TypeRelation<'tcx>, f: Self, g: Self) -> TypeResult<'tcx, Self> {
        let params = relation.relate(f.params, g.params)?;
        let ret = relation.relate(f.ret, g.ret)?;
        Ok(Self { params, ret })
    }
}
