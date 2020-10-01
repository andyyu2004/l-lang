use super::{Expr, NodeId, Pattern, Stmt, StmtKind, Ty, P};
use crate::ir::VariantIdx;
use crate::lexer::{symbol, Symbol, Tok, TokenType};
use crate::span::Span;
use crate::util;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub id: NodeId,
    pub span: Span,
    pub pat: P<Pattern>,
    pub body: P<Expr>,
    pub guard: Option<P<Expr>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Variant {
    pub id: NodeId,
    pub span: Span,
    pub ident: Ident,
    pub kind: VariantKind,
}

/// access of field `p.x`
/// also used in struct expressions
/// `SomeStruct {
///     <ident>: <expr>,
///     ..
/// }`
#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    pub id: NodeId,
    pub span: Span,
    pub ident: Ident,
    pub expr: P<Expr>,
}

/// struct S {
///     x: int, <- field decl
///     y: bool,   <- field decl
/// }
#[derive(Debug, PartialEq, Clone)]
pub struct FieldDecl {
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
    pub ident: Option<Ident>,
    pub ty: P<Ty>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VariantKind {
    Struct(Vec<FieldDecl>),
    Tuple(Vec<FieldDecl>),
    Unit,
}

#[derive(Debug, Copy, Clone, Eq)]
pub struct Ident {
    pub span: Span,
    pub symbol: Symbol,
}

impl Ident {
    pub fn new(span: Span, symbol: Symbol) -> Self {
        Self { span, symbol }
    }

    pub fn unspanned(symbol: Symbol) -> Self {
        Self::new(Span::empty(), symbol)
    }

    pub fn empty() -> Self {
        Self::new(Span::empty(), symbol::EMPTY)
    }

    /// joins two identifiers `a` and `b`
    /// a::b
    pub fn concat_as_path(self, ident: Self) -> Self {
        let mut concatenated = self.as_str().to_owned();
        concatenated.push_str("::");
        concatenated.push_str(ident.as_str());
        let sym = Symbol::intern(&concatenated);
        // it doesn't really make sense to merge the spans
        // so we just take the latter for now
        Ident::new(ident.span, sym)
    }
}

impl From<Span> for Ident {
    fn from(span: Span) -> Self {
        Self { symbol: span.intern(), span }
    }
}

impl Deref for Ident {
    type Target = Symbol;

    fn deref(&self) -> &Self::Target {
        &self.symbol
    }
}

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol.hash(state)
    }
}

/// ignore span in equality checks
impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub span: Span,
    pub id: NodeId,
    pub stmts: Vec<P<Stmt>>,
}

impl Block {
    pub fn any_expr(&self, f: fn(&Expr) -> bool) -> bool {
        self.stmts.iter().any(|stmt| match &stmt.kind {
            StmtKind::Let(l) => l.init.as_ref().map(|expr| expr.any(f)).unwrap_or(false),
            StmtKind::Expr(expr) | StmtKind::Semi(expr) => expr.any(f),
        })
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Generics {
    pub span: Span,
    pub params: Vec<TyParam>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct TyParam {
    pub span: Span,
    pub id: NodeId,
    pub ident: Ident,
    pub default: Option<P<Ty>>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct FnSig {
    pub params: Vec<Param>,
    pub ret_ty: Option<P<Ty>>,
}

impl Display for FnSig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn {} -> {:?}", util::join(&self.params, ", "), self.ret_ty)
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Param {
    pub span: Span,
    pub id: NodeId,
    pub pattern: P<Pattern>,
    pub ty: P<Ty>,
}

impl Display for Param {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!();
        // write!(f, "{}: {}", self.pattern, self.ty)
    }
}

pub type Visibility = Spanned<VisibilityKind>;

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum VisibilityKind {
    Public,
    Private,
}

impl Display for VisibilityKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => write!(f, "pub "),
            Self::Private => write!(f, ""),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub struct Spanned<T> {
    pub span: Span,
    pub node: T,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Path {
    pub id: NodeId,
    pub span: Span,
    pub segments: Vec<PathSegment>,
}

// just to make it `std::mem::take`able
impl Default for Path {
    fn default() -> Self {
        Self { id: NodeId(0), span: Span::new(0, 0), segments: Default::default() }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", util::join(&self.segments, "::"))
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct PathSegment {
    pub ident: Ident,
    pub id: NodeId,
    pub args: Option<GenericArgs>,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct GenericArgs {
    pub span: Span,
    pub args: Vec<P<Ty>>,
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Lit {
    Float(f64),
    Int(i64),
    Bool(bool),
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(d) => write!(f, "{}", d),
            Self::Int(i) => write!(f, "{}", i),
            Self::Bool(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum BinOp {
    Mul,
    Div,
    Add,
    Sub,
    Lt,
    Gt,
    Eq,
    Neq,
    /// bitwise and
    And,
    /// bitwise or
    Or,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Or => write!(f, "&"),
            BinOp::And => write!(f, "|"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Neq => write!(f, "!="),
        }
    }
}

impl From<Tok> for BinOp {
    fn from(t: Tok) -> Self {
        match t.ttype {
            TokenType::Plus => Self::Add,
            TokenType::Minus => Self::Sub,
            TokenType::Star => Self::Mul,
            TokenType::Slash => Self::Div,
            TokenType::Gt => Self::Gt,
            TokenType::Lt => Self::Lt,
            k => panic!("invalid binary operator `{:?}`", k),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum UnaryOp {
    /// -x
    Neg,
    /// !x
    Not,
    /// *x
    Deref,
    /// &x
    Ref,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neg => write!(f, "-"),
            Self::Not => write!(f, "!"),
            Self::Deref => write!(f, "*"),
            Self::Ref => write!(f, "&"),
        }
    }
}

impl From<Tok> for UnaryOp {
    fn from(t: Tok) -> Self {
        match t.ttype {
            TokenType::Minus => Self::Neg,
            TokenType::Not => Self::Not,
            TokenType::Star => Self::Deref,
            TokenType::And => Self::Ref,
            k => panic!("invalid unary operator `{:?}`", k),
        }
    }
}
