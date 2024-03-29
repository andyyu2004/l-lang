#[macro_use]
extern crate serde;

mod arena;
mod def;
mod expr;
mod item;
mod item_visitor;
mod pat;
mod stmt;
mod ty;
mod visit;

use crate as ir;

pub use def::*;
pub use def::*;
pub use expr::{Expr, ExprKind};
pub use item::*;
pub use item_visitor::*;
use lc_ast::{Ident, Visibility};
use lc_index::newtype_index;
use lc_span::Span;
pub use pat::{FieldPat, Pattern, PatternKind};
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
pub use stmt::{Stmt, StmtKind};
pub use ty::*;
pub use ty::*;
pub use visit::*;

/// certain things such as qpaths generalize over both expressions and patterns
pub trait ExprOrPat<'ir> {
    fn span(&self) -> Span;
    fn id(&self) -> ir::Id;
}

impl<'ir> ExprOrPat<'ir> for ir::Expr<'ir> {
    fn span(&self) -> Span {
        self.span
    }

    fn id(&self) -> ir::Id {
        self.id
    }
}

impl<'ir> ExprOrPat<'ir> for ir::Pattern<'ir> {
    fn span(&self) -> Span {
        self.span
    }

    fn id(&self) -> ir::Id {
        self.id
    }
}

/// top level IR ast
#[derive(Debug)]
pub struct Ir<'ir> {
    /// DefId of the entry/main function
    pub entry_id: Option<DefId>,
    pub items: BTreeMap<DefId, &'ir ir::Item<'ir>>,
    pub impl_items: BTreeMap<ImplItemId, &'ir ir::ImplItem<'ir>>,
    pub trait_items: BTreeMap<TraitItemId, &'ir ir::TraitItem<'ir>>,
}

newtype_index!(
    #[derive(Serialize, Deserialize)]
    pub struct PkgId {
        DEBUG_FORMAT = "{}",
        const LOCAL_PKG_ID = 0,
    }
);

// #[derive(Debug, Clone, Eq, PartialEq, Copy, Hash)]
// pub struct DefId {
//     pub pkg: PkgId,
//     pub idx: DefIdx,
// }

newtype_index!(
    #[derive(Serialize, Deserialize)]
    pub struct DefId {
        DEBUG_FORMAT = "{}"
    }
);

newtype_index!(
    #[derive(Serialize, Deserialize)]
    pub struct LocalId {
        DEBUG_FORMAT = "{}"
    }
);

newtype_index!(
    #[derive(Serialize, Deserialize)]
    pub struct ParamIdx {
        DEBUG_FORMAT ="{}"
    }
);

newtype_index!(
    #[derive(Serialize, Deserialize)]
    pub struct VariantIdx {
        DEBUG_FORMAT = "VariantIdx({})"
    }
);

newtype_index!(
    #[derive(Serialize, Deserialize)]
    pub struct FieldIdx {
        DEBUG_FORMAT = "{}"
    }
);

impl DefId {
    pub fn dummy() -> Self {
        Self::MAX
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct TraitItemId(pub DefId);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ImplItemId(pub DefId);

impl Display for LocalId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for DefId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id {
    /// id of the immediately enclosing item
    pub def: DefId,
    /// id of node relative to the enclosing def_id
    pub local: LocalId,
}

impl Id {
    pub fn dummy() -> Self {
        Self { def: DefId::dummy(), local: LocalId::MAX }
    }
}
impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}.{:?}", self.def, self.local)
    }
}

#[derive(Debug, Clone)]
pub struct Variant<'ir> {
    pub id: ir::Id,
    pub ident: Ident,
    pub span: Span,
    /// `DefId` of the adt that this variant belongs to
    pub adt_def_id: DefId,
    /// index of the variant in the enum
    pub idx: VariantIdx,
    pub kind: VariantKind<'ir>,
}

#[derive(Debug, Copy, Clone)]
pub enum VariantKind<'ir> {
    Struct(&'ir [ir::FieldDecl<'ir>]),
    Tuple(&'ir [ir::FieldDecl<'ir>]),
    Unit,
}

impl<'ir> VariantKind<'ir> {
    pub fn is_tuple(&self) -> bool {
        matches!(self, VariantKind::Tuple(_))
    }
}

impl<'ir> From<&VariantKind<'ir>> for ir::CtorKind {
    fn from(kind: &VariantKind<'ir>) -> Self {
        match kind {
            VariantKind::Struct(..) => Self::Struct,
            VariantKind::Tuple(..) => Self::Tuple,
            VariantKind::Unit => Self::Unit,
        }
    }
}

impl<'ir> VariantKind<'ir> {
    pub fn fields(&self) -> &'ir [ir::FieldDecl<'ir>] {
        match self {
            Self::Struct(fields) | Self::Tuple(fields) => fields,
            Self::Unit => &[],
        }
    }
}

#[derive(Debug)]
pub struct Field<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub ident: Ident,
    pub expr: &'ir ir::Expr<'ir>,
}

#[derive(Debug)]
pub struct FieldDecl<'ir> {
    pub span: Span,
    pub ident: Ident,
    pub vis: Visibility,
    pub id: ir::Id,
    pub ty: &'ir ir::Ty<'ir>,
}

impl Display for ParamIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Generics<'ir> {
    pub span: Span,
    pub params: &'ir [ir::TyParam<'ir>],
}

#[derive(Debug)]
pub struct GenericArgs<'ir> {
    pub span: Span,
    pub args: &'ir [ir::Ty<'ir>],
}

#[derive(Debug)]
pub struct TyParam<'ir> {
    pub span: Span,
    pub id: ir::Id,
    pub ident: Ident,
    pub index: ParamIdx,
    pub default: Option<&'ir ir::Ty<'ir>>,
}

#[derive(Debug)]
pub struct Body<'ir> {
    pub params: &'ir [ir::Param<'ir>],
    pub expr: &'ir ir::Expr<'ir>,
}

impl<'ir> Body<'ir> {
    pub fn id(&self) -> ir::Id {
        self.expr.id
    }
}

#[derive(Debug)]
pub enum MatchSource {
    Match,
    If,
}

#[derive(Debug)]
pub struct Arm<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: &'ir ir::Pattern<'ir>,
    pub guard: Option<&'ir ir::Expr<'ir>>,
    pub body: &'ir ir::Expr<'ir>,
}

#[derive(Debug)]
pub struct FnSig<'ir> {
    // rest of parameter information is in `Body`
    pub inputs: &'ir [ir::Ty<'ir>],
    pub output: Option<&'ir ir::Ty<'ir>>,
}

/// qualified path
#[derive(Debug, Clone)]
pub enum QPath<'ir> {
    Resolved(&'ir Path<'ir>),
    TypeRelative(&'ir ir::Ty<'ir>, &'ir PathSegment<'ir>),
}

impl<'ir> QPath<'ir> {
    pub fn span(&self) -> Span {
        match self {
            QPath::Resolved(path) => path.span,
            QPath::TypeRelative(ty, _) => ty.span,
        }
    }
}

impl<'ir> Display for QPath<'ir> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            QPath::Resolved(path) => write!(f, "{}", path),
            QPath::TypeRelative(ty, segment) => write!(f, "<{}>::{}", ty, segment),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Path<'ir> {
    pub span: Span,
    pub res: Res,
    pub segments: &'ir [PathSegment<'ir>],
}

impl<'ir> Path<'ir> {
    pub fn is_enum_ctor(&self) -> bool {
        matches!(self.res, Res::Def(_, DefKind::Ctor(..)))
    }
}

impl<'ir> Display for Path<'ir> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", lc_util::join2(self.segments, "::"))
    }
}

#[derive(Debug)]
pub struct Param<'ir> {
    // no type as it is in `FnSig`
    pub span: Span,
    pub id: ir::Id,
    pub pat: &'ir ir::Pattern<'ir>,
}

#[derive(Debug, Clone)]
pub struct PathSegment<'ir> {
    pub ident: Ident,
    pub args: Option<&'ir ir::GenericArgs<'ir>>,
}

impl<'ir> Display for PathSegment<'ir> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident)
    }
}

#[derive(Debug)]
pub struct Block<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub is_unsafe: bool,
    pub stmts: &'ir [ir::Stmt<'ir>],
    pub expr: Option<&'ir ir::Expr<'ir>>,
}

pub enum Lvalue {
    Local(ir::Id),
}

#[derive(Debug)]
pub struct Let<'ir> {
    pub id: ir::Id,
    pub span: Span,
    pub pat: &'ir ir::Pattern<'ir>,
    pub ty: Option<&'ir ir::Ty<'ir>>,
    pub init: Option<&'ir ir::Expr<'ir>>,
}
