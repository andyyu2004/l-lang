mod adjustments;
mod instance;
mod list;
mod relate;
mod substs;
mod tables;
mod tcx;
mod traverse;
mod type_error;

pub use adjustments::{Adjuster, Adjustment, AdjustmentKind};
pub use instance::{Instance, InstanceId, InstanceKind};
pub use list::List;
pub use relate::{Relate, TypeRelation};
pub use substs::*;
pub use tables::TypeckTables;
pub use tcx::{tls, GlobalCtx, TyCtx};
pub use traverse::*;
pub use type_error::{TypeError, TypeResult};
pub use InferTy::*;
pub use TyKind::*;

use ast::{Ident, Mutability, Visibility};
use bitflags::bitflags;
use index::{Idx, IndexVec};
use ir::{self, CtorKind, DefId, FieldIdx, ParamIdx, Res, VariantIdx};
use span::Span;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;
use util;

pub type Ty<'tcx> = &'tcx TyS<'tcx>;

#[derive(Debug, Eq)]
pub struct TyS<'tcx> {
    pub flags: TyFlags,
    pub kind: TyKind<'tcx>,
}

impl<'tcx> TyS<'tcx> {
    pub fn is_unit(&self) -> bool {
        match self.kind {
            TyKind::Tuple(tys) => tys.is_empty(),
            _ => false,
        }
    }
}

/// visitor that searches for a specific type variables (for the occurs check)
struct TyVidVisitor {
    tyvid: TyVid,
}

/// type variable id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct TyVid {
    pub index: u32,
}

impl Display for TyVid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "?{}", self.index)
    }
}

impl<'tcx> TypeVisitor<'tcx> for TyVidVisitor {
    fn visit_ty(&mut self, ty: Ty<'tcx>) -> bool {
        match ty.kind {
            TyKind::Infer(InferTy::TyVar(v)) => v == self.tyvid,
            _ => ty.inner_visit_with(self),
        }
    }
}

impl<'tcx> TyS<'tcx> {
    pub fn contains_tyvid(&self, tyvid: TyVid) -> bool {
        self.visit_with(&mut TyVidVisitor { tyvid })
    }

    pub fn expect_scheme(&self) -> (Generics<'tcx>, Ty<'tcx>) {
        match self.kind {
            TyKind::Scheme(forall, ty) => (forall, ty),
            _ => panic!("expected TyKind::Scheme, found {}", self),
        }
    }

    pub fn expect_tuple(&self) -> SubstsRef<'tcx> {
        match self.kind {
            TyKind::Tuple(tys) => tys,
            _ => panic!("expected TyKind::Tuple, found {}", self),
        }
    }

    pub fn expect_fn(&self) -> (SubstsRef<'tcx>, Ty<'tcx>) {
        match self.kind {
            TyKind::Fn(params, ret) => (params, ret),
            _ => panic!("expected TyKind::Fn, found {}", self),
        }
    }

    pub fn is_box(&self) -> bool {
        match self.kind {
            TyKind::Box(..) => true,
            _ => false,
        }
    }

    pub fn deref_ty(&self) -> Ty<'tcx> {
        match self.kind {
            TyKind::Box(_, ty) | TyKind::Ptr(ty) => ty,
            _ => panic!("cannot dereference a non-pointer type"),
        }
    }

    pub fn expect_adt(&self) -> (&'tcx AdtTy<'tcx>, SubstsRef<'tcx>) {
        match self.kind {
            TyKind::Adt(adt, substs) => (adt, substs),
            _ => panic!("expected TyKind::Adt, found {}", self),
        }
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

// #[derive(Debug)]
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub enum TyKind<'tcx> {
    /// bool
    Bool,
    /// the integer type of the discriminant (currently i16)
    Discr,
    /// char
    Char,
    /// float
    Float,
    /// Int
    Int,
    Error,
    Never,
    /// [<ty>; n]
    Array(Ty<'tcx>, usize),
    /// fn(<ty>...) -> <ty>
    Fn(SubstsRef<'tcx>, Ty<'tcx>),
    Tuple(SubstsRef<'tcx>),
    Infer(InferTy),
    Ptr(Ty<'tcx>),
    Param(ParamTy),
    Adt(&'tcx AdtTy<'tcx>, SubstsRef<'tcx>),
    Scheme(Generics<'tcx>, Ty<'tcx>),
    /// pointer to a type
    /// created by box expressions
    /// mutability inherited by the pointee?
    /// x: T -> box x: &T
    /// mut x: T -> box x: &mut T
    Box(Mutability, Ty<'tcx>),
    Opaque(DefId, SubstsRef<'tcx>),
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct Generics<'tcx> {
    pub params: &'tcx [TyParam<'tcx>],
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct TyParam<'tcx> {
    pub id: ir::Id,
    pub span: Span,
    pub ident: Ident,
    pub index: ParamIdx,
    pub default: Option<Ty<'tcx>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Projection<'tcx> {
    Deref,
    /// the type is the type of the entire expression after projection
    /// struct S { x: int }
    /// let s: S;
    /// s.x :: int
    /// so the projection from `s` would be `Projection::Field(0, int)`
    Field(FieldIdx, Ty<'tcx>),
    PointerCast(Ty<'tcx>),
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum AdtKind {
    Struct,
    Enum,
}

#[derive(Debug, Eq, Hash, Clone)]
pub struct AdtTy<'tcx> {
    pub def_id: DefId,
    pub kind: AdtKind,
    pub ident: Ident,
    pub variants: IndexVec<VariantIdx, VariantTy<'tcx>>,
}

impl<'tcx> PartialEq for AdtTy<'tcx> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl<'tcx> AdtTy<'tcx> {
    pub fn single_variant(&self) -> &VariantTy<'tcx> {
        assert_eq!(self.variants.len(), 1);
        &self.variants[VariantIdx::new(0)]
    }

    pub fn variant_idx_with_res(&self, res: Res) -> VariantIdx {
        match res {
            Res::Def(def, ..) => self.variant_idx_with_ctor(def),
            _ => unreachable!(),
        }
    }

    pub fn variant_idx_with_ctor(&self, ctor_id: DefId) -> VariantIdx {
        self.variants.iter_enumerated().find(|(_, v)| v.ctor == Some(ctor_id)).unwrap().0
    }

    // find the variant who has the constructor that matches the `ctor_id`
    pub fn variant_with_ctor(&self, ctor_id: DefId) -> &VariantTy<'tcx> {
        self.variants.iter().find(|v| v.ctor == Some(ctor_id)).unwrap()
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct VariantTy<'tcx> {
    pub ident: Ident,
    /// `None` for struct variants
    pub ctor: Option<DefId>,
    pub ctor_kind: CtorKind,
    pub fields: &'tcx [FieldTy<'tcx>],
}

#[derive(Debug)]
pub struct FieldTy<'tcx> {
    pub def_id: DefId,
    pub ident: Ident,
    pub vis: Visibility,
    pub ir_ty: &'tcx ir::Ty<'tcx>,
}

impl<'tcx> Eq for FieldTy<'tcx> {
}

impl<'tcx> Hash for FieldTy<'tcx> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.def_id.hash(state);
        self.ident.hash(state);
        self.vis.hash(state);
    }
}

impl<'tcx> PartialEq for FieldTy<'tcx> {
    fn eq(&self, other: &Self) -> bool {
        self.def_id == other.def_id && self.ident == other.ident && self.vis == other.vis
    }
}

bitflags! {
    pub struct TyFlags: u32  {
        const HAS_ERROR = 1 << 0;
        const HAS_INFER = 1 << 1;
        const HAS_PARAM = 1 << 2;
    }
}

pub trait TyFlag {
    fn ty_flags(&self) -> TyFlags;
}

impl<'tcx> TyFlag for TyS<'tcx> {
    fn ty_flags(&self) -> TyFlags {
        self.kind.ty_flags()
    }
}

impl<'tcx> TyFlag for SubstsRef<'tcx> {
    fn ty_flags(&self) -> TyFlags {
        self.iter().fold(TyFlags::empty(), |acc, ty| acc | ty.ty_flags())
    }
}

impl<'tcx> TyS<'tcx> {
    pub fn has_flags(&self, flags: TyFlags) -> bool {
        self.flags.intersects(flags)
    }

    pub fn has_infer_vars(&self) -> bool {
        self.has_flags(TyFlags::HAS_INFER)
    }

    pub fn has_ty_params(&self) -> bool {
        self.has_flags(TyFlags::HAS_PARAM)
    }

    pub fn contains_err(&self) -> bool {
        self.has_flags(TyFlags::HAS_ERROR)
    }
}

impl<'tcx> TyFlag for TyKind<'tcx> {
    fn ty_flags(&self) -> TyFlags {
        match self {
            TyKind::Fn(params, ret) => params.ty_flags() | ret.ty_flags(),
            TyKind::Opaque(_, tys) | TyKind::Tuple(tys) => tys.ty_flags(),
            TyKind::Infer(..) => TyFlags::HAS_INFER,
            TyKind::Param(..) => TyFlags::HAS_PARAM,
            TyKind::Error => TyFlags::HAS_ERROR,
            TyKind::Adt(_, substs) => substs.ty_flags(),
            TyKind::Ptr(ty) | TyKind::Array(ty, _) | TyKind::Scheme(_, ty) | TyKind::Box(_, ty) =>
                ty.ty_flags(),
            TyKind::Discr
            | TyKind::Float
            | TyKind::Never
            | TyKind::Bool
            | TyKind::Char
            | TyKind::Int => TyFlags::empty(),
        }
    }
}

impl<'tcx> Debug for TyKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'tcx> Display for TyKind<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TyKind::Box(m, ty) => write!(f, "&{}{}", m, ty),
            TyKind::Fn(params, ret) =>
                write!(f, "fn({})->{}", util::join2(params.into_iter(), ","), ret),
            TyKind::Infer(infer_ty) => write!(f, "{}", infer_ty),
            TyKind::Ptr(ty) => write!(f, "*{}", ty),
            TyKind::Array(ty, n) => write!(f, "[{};{}]", ty, n),
            TyKind::Tuple(tys) => write!(f, "({})", tys),
            TyKind::Param(param_ty) => write!(f, "{}", param_ty),
            TyKind::Scheme(forall, ty) => write!(f, "∀{}.{}", forall, ty),
            TyKind::Adt(adt, substs) => write!(f, "{}<{}>", adt.ident, substs),
            TyKind::Opaque(_, _) => write!(f, "opaque"),
            TyKind::Bool => write!(f, "bool"),
            TyKind::Char => write!(f, "char"),
            TyKind::Int => write!(f, "int"),
            TyKind::Float => write!(f, "float"),
            TyKind::Error => write!(f, "err"),
            TyKind::Never => write!(f, "!"),
            TyKind::Discr => write!(f, "discr"),
        }
    }
}

impl<'tcx> Display for TyParam<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident)
    }
}

impl<'tcx> Display for Generics<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "τ{}", util::join2(self.params.iter(), ","))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParamTy {
    pub def_id: DefId,
    pub idx: ParamIdx,
}

impl Display for ParamTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "τ{}", self.idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InferTy {
    TyVar(TyVid),
}

impl Display for InferTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TyVar(vid) => write!(f, "{}", vid),
        }
    }
}

/// upvars are identified by the closure that references them as well as the original variable id
/// the original variable id alone is not sufficient as multiple closures can reference the same
/// variable and the UpvarId would not be unique
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct UpvarId {
    pub closure_id: ir::Id,
    pub var_id: ir::Id,
}

impl<'tcx> Display for TyS<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

/// typed constant value
#[derive(Debug, Hash, PartialEq, Clone, Eq)]
pub struct Const<'tcx> {
    pub kind: ConstKind,
    pub ty: Ty<'tcx>,
}

impl<'tcx> Const<'tcx> {
    pub fn new(kind: ConstKind, ty: Ty<'tcx>) -> Self {
        Self { kind, ty }
    }

    pub fn unit(tcx: TyCtx<'tcx>) -> Self {
        Self::new(ConstKind::Unit, tcx.types.unit)
    }
}

#[derive(Debug, Clone)]
pub enum ConstKind {
    Float(f64),
    Int(i64),
    Bool(bool),
    Unit,
}

// maybe this is a bad idea due to the presence of an f64?
impl Eq for ConstKind {
}

impl PartialEq for ConstKind {
    fn eq(&self, other: &Self) -> bool {
        use ConstKind::*;
        match (self, other) {
            (Float(a), Float(b)) => a.to_bits() == b.to_bits(),
            (Int(i), Int(j)) => i == j,
            (Bool(b), Bool(c)) => b == c,
            (Unit, Unit) => true,
            _ => false,
        }
    }
}

impl std::hash::Hash for ConstKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ConstKind::Float(f) => f.to_bits().hash(state),
            ConstKind::Int(i) => i.hash(state),
            ConstKind::Bool(b) => b.hash(state),
            ConstKind::Unit => {}
        };
    }
}

impl<'tcx> Display for Const<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ConstKind::Float(d) => write!(f, "{:?}", d),
            ConstKind::Int(i) => write!(f, "{}", i),
            ConstKind::Bool(b) => write!(f, "{}", b),
            ConstKind::Unit => write!(f, "()"),
        }
    }
}
