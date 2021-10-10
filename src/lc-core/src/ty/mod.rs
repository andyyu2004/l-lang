mod adjustments;
mod codec;
mod instance;
mod list;
mod relate;
mod substs;
mod tables;
mod tcx;
mod traverse;
mod type_error;

pub use adjustments::{Adjuster, Adjustment, AdjustmentKind, PointerCast};
use ena::unify::UnifyKey;
pub use instance::{Instance, InstanceKind, Instances};
pub use list::List;
pub use relate::{Relate, TypeRelation};
pub use substs::*;
pub use tables::TypeckTables;
pub use tcx::{tls, GlobalCtx, TyCtx};
pub use traverse::*;
pub use type_error::{TypeError, TypeResult};
pub use InferTy::*;
pub use TyKind::*;

use crate::queries::Queries;
use lc_ast::{Ident, Visibility};
use bitflags::bitflags;
use ir::{self, CtorKind, DefId, FieldIdx, ParamIdx, Res, VariantIdx};
use lc_index::{Idx, IndexVec};
use lc_span::Span;
use rustc_hash::FxHashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr;

pub(crate) fn provide(queries: &mut Queries) {
    instance::provide(queries);
}

pub type Ty<'tcx> = &'tcx Type<'tcx>;

#[derive(Debug, Eq, Serialize)]
pub struct Type<'tcx> {
    pub flags: TyFlags,
    pub kind: TyKind<'tcx>,
}

impl<'tcx> Type<'tcx> {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct TyVid {
    pub index: u32,
}

impl UnifyKey for TyVid {
    type Value = ();

    fn index(&self) -> u32 {
        self.index
    }

    fn from_index(i: u32) -> TyVid {
        TyVid { index: i }
    }

    fn tag() -> &'static str {
        "TyVid"
    }
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

impl<'tcx> Type<'tcx> {
    pub fn contains_tyvid(&self, tyvid: TyVid) -> bool {
        self.visit_with(&mut TyVidVisitor { tyvid })
    }

    pub fn expect_tuple(&self) -> SubstsRef<'tcx> {
        match self.kind {
            TyKind::Tuple(tys) => tys,
            _ => panic!("expected TyKind::Tuple, found {}", self),
        }
    }

    pub fn expect_fn_ptr(&self) -> FnSig<'tcx> {
        match self.kind {
            TyKind::FnPtr(fn_sig) => fn_sig,
            _ => panic!("expected TyKind::FnPtr, found {}", self),
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
            TyKind::Box(ty) | TyKind::Ptr(ty) => ty,
            _ => panic!("cannot dereference a non-pointer type"),
        }
    }

    pub fn expect_adt(&self) -> (&'tcx AdtTy, SubstsRef<'tcx>) {
        match self.kind {
            TyKind::Adt(adt, substs) => (adt, substs),
            _ => panic!("expected TyKind::Adt, found {}", self),
        }
    }
}

impl<'tcx> Hash for Type<'tcx> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self as *const Type<'tcx>).hash(state)
    }
}

/// we can perform equality using pointers as we ensure that at most one of each TyS is allocated
/// (by doing a deep compare on TyKind during allocation)
impl<'tcx> PartialEq for Type<'tcx> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Serialize)]
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
    /// box pointer to a type
    /// created by box expressions
    /// x: T => box x: &T
    Box(Ty<'tcx>),
    /// fn(<ty>...) -> <ty>
    FnPtr(FnSig<'tcx>),
    /// [<ty>; n]
    Array(Ty<'tcx>, usize),
    /// (T, U, V, ...)
    /// the `SubstsRef` is to be treated as a list of types
    /// not as a substitution itself
    Tuple(SubstsRef<'tcx>),
    Infer(InferTy),
    Ptr(Ty<'tcx>),
    Param(ParamTy),
    Opaque(DefId, SubstsRef<'tcx>),
    Adt(&'tcx AdtTy, SubstsRef<'tcx>),
}

/// this is the type-level representation of the type of a function
#[derive(Eq, Hash, PartialEq, Clone, Copy, Serialize)]
pub struct FnSig<'tcx> {
    pub params: SubstsRef<'tcx>,
    pub ret: Ty<'tcx>,
}

impl<'tcx> Display for FnSig<'tcx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn({})->{}", lc_util::join2(self.params.into_iter(), ","), self.ret)
    }
}

#[derive(Eq, Hash, PartialEq, Debug)]
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

#[derive(Debug, Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum AdtKind {
    Struct,
    Enum,
}

#[derive(Debug, Eq, Hash, Serialize, Deserialize)]
pub struct AdtTy {
    pub def_id: DefId,
    pub kind: AdtKind,
    pub ident: Ident,
    pub variants: IndexVec<VariantIdx, VariantTy>,
}

impl AdtTy {
    pub fn is_enum(&self) -> bool {
        self.kind == AdtKind::Enum
    }
}

impl PartialEq for AdtTy {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self, other)
    }
}

impl AdtTy {
    pub fn single_variant(&self) -> &VariantTy {
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
        self.variants.iter_enumerated().find(|(_, v)| v.def_id == ctor_id).unwrap().0
    }

    // find the variant who has the constructor that matches the `ctor_id`
    pub fn variant_with_ctor(&self, ctor_id: DefId) -> &VariantTy {
        let idx = self.variant_idx_with_ctor(ctor_id);
        &self.variants[idx]
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub struct VariantTy {
    pub def_id: DefId,
    pub ident: Ident,
    pub ctor_kind: CtorKind,
    pub fields: Vec<FieldTy>,
}

/// the type representation of a field
/// the `ty::Ty` can be found using `type_of` with this def_id
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct FieldTy {
    pub def_id: DefId,
    pub ident: Ident,
    pub vis: Visibility,
}

impl FieldTy {
    /// type of the field
    // we require this indirection instead of storing `ty: Ty` directly as a field
    // because fields may refer to the the struct/enum that it is declared in
    // therefore, the lowering must be done post type collection
    pub fn ty<'tcx>(&self, tcx: TyCtx<'tcx>, substs: SubstsRef<'tcx>) -> Ty<'tcx> {
        // TODO cache this result somewhere?
        let ty = tcx.type_of(self.def_id);
        ty.subst(tcx, substs)
    }
}

bitflags! {
    #[derive(Serialize)]
    pub struct TyFlags: u32  {
        const HAS_ERROR = 1 << 0;
        const HAS_INFER = 1 << 1;
        const HAS_PARAM = 1 << 2;
    }
}

pub trait TyFlag {
    fn ty_flags(&self) -> TyFlags;
}

impl<'tcx> TyFlag for Type<'tcx> {
    fn ty_flags(&self) -> TyFlags {
        self.kind.ty_flags()
    }
}

impl<'tcx> TyFlag for FnSig<'tcx> {
    fn ty_flags(&self) -> TyFlags {
        self.params.ty_flags() | self.ret.ty_flags()
    }
}

impl<'tcx> TyFlag for SubstsRef<'tcx> {
    fn ty_flags(&self) -> TyFlags {
        self.iter().fold(TyFlags::empty(), |acc, ty| acc | ty.ty_flags())
    }
}

pub trait HasTyFlags {
    fn has_flags(&self, flags: TyFlags) -> bool;

    fn has_infer_vars(&self) -> bool {
        self.has_flags(TyFlags::HAS_INFER)
    }

    fn has_ty_params(&self) -> bool {
        self.has_flags(TyFlags::HAS_PARAM)
    }

    fn contains_err(&self) -> bool {
        self.has_flags(TyFlags::HAS_ERROR)
    }
}

impl<'tcx> HasTyFlags for Type<'tcx> {
    fn has_flags(&self, flags: TyFlags) -> bool {
        self.flags.intersects(flags)
    }
}

impl<'tcx> HasTyFlags for SubstsRef<'tcx> {
    fn has_flags(&self, flags: TyFlags) -> bool {
        self.iter().any(|ty| ty.has_flags(flags))
    }
}

impl<'tcx> TyFlag for TyKind<'tcx> {
    fn ty_flags(&self) -> TyFlags {
        match self {
            TyKind::FnPtr(sig) => sig.ty_flags(),
            TyKind::Opaque(_, tys) | TyKind::Tuple(tys) => tys.ty_flags(),
            TyKind::Infer(..) => TyFlags::HAS_INFER,
            TyKind::Param(..) => TyFlags::HAS_PARAM,
            TyKind::Adt(_, substs) => substs.ty_flags(),
            TyKind::Ptr(ty) | TyKind::Array(ty, _) | TyKind::Box(ty) => ty.ty_flags(),
            TyKind::Discr
            | TyKind::Float
            | TyKind::Never
            | TyKind::Bool
            | TyKind::Char
            | TyKind::Int => TyFlags::empty(),
            TyKind::Error => TyFlags::HAS_ERROR,
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
            TyKind::Box(ty) => write!(f, "&{}", ty),
            TyKind::Ptr(ty) => write!(f, "*{}", ty),
            TyKind::FnPtr(sig) => write!(f, "{}", sig),
            TyKind::Infer(infer_ty) => write!(f, "{}", infer_ty),
            TyKind::Array(ty, n) => write!(f, "[{};{}]", ty, n),
            TyKind::Tuple(tys) => write!(f, "({})", tys),
            TyKind::Param(param_ty) => write!(f, "{}", param_ty),
            TyKind::Adt(adt, substs) => write!(f, "{}<{}>", adt.ident, substs),
            TyKind::Opaque(_, _) => write!(f, "opaque"),
            TyKind::Bool => write!(f, "bool"),
            TyKind::Char => write!(f, "char"),
            TyKind::Int => write!(f, "int"),
            TyKind::Float => write!(f, "float"),
            TyKind::Never => write!(f, "!"),
            TyKind::Discr => write!(f, "discr"),
            TyKind::Error => write!(f, "err"),
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
        write!(f, "{}", lc_util::join2(self.params.iter(), ","))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParamTy {
    pub def_id: DefId,
    pub idx: ParamIdx,
    pub ident: Ident,
}

impl Ord for ParamTy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl PartialOrd for ParamTy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl Display for ParamTy {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl<'tcx> Display for Type<'tcx> {
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

// deriving partialeq even with custom hash impl should be fine
// just avoid the nasty NaN business
#[derive(Debug, Clone, PartialEq)]
pub enum ConstKind {
    Float(f64),
    Int(i64),
    Discr(i16),
    Bool(bool),
    Unit,
}

// maybe this is a bad idea due to the presence of an f64?
impl Eq for ConstKind {
}

impl std::hash::Hash for ConstKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ConstKind::Float(f) => f.to_bits().hash(state),
            ConstKind::Discr(d) => d.hash(state),
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
            ConstKind::Discr(d) => write!(f, "{}", d),
            ConstKind::Bool(b) => write!(f, "{}", b),
            ConstKind::Unit => write!(f, "()"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct InherentImpls {
    pub inherent_impls: FxHashMap<DefId, Vec<DefId>>,
}

#[derive(Clone, Debug, Default)]
pub struct TraitImpls {
    pub trait_impls: FxHashMap<DefId, Vec<DefId>>,
}
