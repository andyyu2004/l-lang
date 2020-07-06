use crate::typeck::inference::TyVid;
use std::fmt::{self, Display, Formatter};
use std::{
    hash::{Hash, Hasher}, marker::PhantomData, ptr
};

crate type Ty<'tcx> = &'tcx TyS<'tcx>;

#[derive(Debug, Eq)]
crate struct TyS<'tcx> {
    pub kind: TyKind<'tcx>,
}

impl<'tcx> Hash for TyS<'tcx> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as *const TyS<'tcx>).hash(state)
    }
}

/// we can perform equality using pointers as we ensure that at most one of each TyS is allocated
impl<'tcx> PartialEq for TyS<'tcx> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
crate enum TyKind<'tcx> {
    _Phantom(&'tcx PhantomData<()>),
    Bool,
    Unit,
    Char,
    Num,
    Infer(InferTy),
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

impl<'tcx> Display for TyKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Unit => write!(f, "()"),
            Self::Char => write!(f, "char"),
            Self::Num => write!(f, "num"),
            Self::Infer(x) => write!(f, "{:?}", x),
            _ => todo!(),
        }
    }
}
