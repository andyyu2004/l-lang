use crate::ty::SubstRef;
use crate::ty::{TypeFoldable, TypeVisitor};
use crate::{typeck::inference::TyVid, util};
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;

crate type Ty<'tcx> = &'tcx TyS<'tcx>;

#[derive(Debug, Eq)]
crate struct TyS<'tcx> {
    pub kind: TyKind<'tcx>,
}

/// visitor that searches for inference variables
struct InferenceVarVisitor;

impl<'tcx> TypeVisitor<'tcx> for InferenceVarVisitor {
    fn visit_ty(&mut self, ty: Ty<'tcx>) -> bool {
        match ty.kind {
            TyKind::Infer(_) => true,
            _ => ty.inner_visit_with(self),
        }
    }
}

impl<'tcx> TyS<'tcx> {
    pub fn expect_fn(&self) -> (SubstRef<'tcx>, Ty<'tcx>) {
        match self.kind {
            TyKind::Fn(params, ret) => (params, ret),
            _ => panic!("expected TyKind::Fn, found {}", self),
        }
    }

    /// returns true if type contains inference variables
    pub fn has_infer_vars(&self) -> bool {
        self.visit_with(&mut InferenceVarVisitor)
    }
}

impl<'tcx> Hash for TyS<'tcx> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as *const TyS<'tcx>).hash(state)
    }
}

/// we can perform equality using pointers as we ensure that at most one of each TyS is allocated
/// (by doing a deep compare on TyKind during allocation)
impl<'tcx> PartialEq for TyS<'tcx> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
crate enum TyKind<'tcx> {
    /// bool
    Bool,
    /// ()
    Unit,
    /// char
    Char,
    /// number
    Num,
    /// [<ty>]
    Array(Ty<'tcx>),
    /// fn(<ty>...) -> <ty>
    Fn(SubstRef<'tcx>, Ty<'tcx>),
    Tuple(SubstRef<'tcx>),
    Infer(InferTy),
}

impl<'tcx> Display for TyKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TyKind::Bool => write!(f, "bool"),
            TyKind::Unit => write!(f, "()"),
            TyKind::Char => write!(f, "char"),
            TyKind::Num => write!(f, "number"),
            TyKind::Fn(params, ret) =>
                write!(f, "({})->{}", util::join2(params.into_iter(), ","), ret),
            TyKind::Infer(infer) => write!(f, "{:?}", infer),
            TyKind::Array(ty) => write!(f, "[{}]", ty),
            TyKind::Tuple(tys) => write!(f, "({})", tys),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
crate enum InferTy {
    TyVar(TyVid),
    // IntVar(IntVid),
    // FloatVar(FloatVid),
}

impl<'tcx> Display for TyS<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}
