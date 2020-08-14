use super::{Expr, NodeId, Pattern, Stmt, Ty, P};
use crate::lexer::{Symbol, Tok, TokenType};
use crate::span::Span;
use crate::util;
use std::fmt::{self, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Clone)]
pub struct Variant {
    pub id: NodeId,
    pub span: Span,
    pub vis: Visibility,
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
///     x: number, <- field decl
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

impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol.hash(state)
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol
    }
}

/// we don't have access to the actual identifier without further context
/// so simply display it as $i where `i` is the symbol index
impl Display for Ident {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol.0)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub span: Span,
    pub id: NodeId,
    pub stmts: Vec<P<Stmt>>,
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
    pub inputs: Vec<Param>,
    pub output: Option<P<Ty>>,
}

impl Display for FnSig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "fn {} -> {:?}", util::join(&self.inputs, ", "), self.output)
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
    pub span: Span,
    pub segments: Vec<PathSegment>,
}

// just to make it `std::mem::take` able
impl Default for Path {
    fn default() -> Self {
        Self { span: Span::new(0, 0), segments: Default::default() }
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
    pub args: Option<()>,
}

impl Display for PathSegment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ident)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Lit {
    Num(f64),
    Bool(bool),
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Num(i) => write!(f, "{}", i),
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
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
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
            k => panic!("Invalid binary operator `{:?}`", k),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Neg => write!(f, "-"),
            Self::Not => write!(f, "!"),
        }
    }
}

impl From<Tok> for UnaryOp {
    fn from(t: Tok) -> Self {
        match t.ttype {
            TokenType::Minus => Self::Neg,
            TokenType::Not => Self::Not,
            k => panic!("Invalid unary operator `{:?}`", k),
        }
    }
}
