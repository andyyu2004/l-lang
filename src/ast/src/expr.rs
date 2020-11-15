use super::*;
use span::Span;
use std::fmt::{self, Display, Formatter};
use util;

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub span: Span,
    pub id: NodeId,
    pub kind: ExprKind,
}

/// this is only used to make expr `takeable`
impl Default for Expr {
    fn default() -> Self {
        Self { span: Span::default(), id: NodeId::new(0), kind: ExprKind::Lit(Lit::Int(0)) }
    }
}

impl Expr {
    pub fn is_named_closure(&self) -> bool {
        match self.kind {
            ExprKind::Closure(name, ..) => name.is_some(),
            _ => false,
        }
    }
}

impl Expr {
    pub fn new(span: Span, id: NodeId, kind: ExprKind) -> Self {
        Self { span, id, kind }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Lit(Lit),
    Bin(BinOp, P<Expr>, P<Expr>),
    Unary(UnaryOp, P<Expr>),
    Paren(P<Expr>),
    Block(P<Block>),
    Loop(P<Block>),
    While(P<Expr>, P<Block>),
    Path(Path),
    Tuple(Vec<P<Expr>>),
    Ret(Option<P<Expr>>),
    Assign(P<Expr>, P<Expr>),
    Closure(Option<Ident>, FnSig, P<Expr>),
    Call(P<Expr>, Vec<P<Expr>>),
    If(P<Expr>, P<Block>, Option<P<Expr>>),
    Struct(Path, Vec<Field>),
    Field(P<Expr>, Ident),
    Box(P<Expr>),
    Match(P<Expr>, Vec<Arm>),
    Err,
}

impl Display for ExprKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Box(expr) => write!(fmt, "box {}", expr),
            Self::Lit(lit) => write!(fmt, "{}", lit),
            Self::Bin(op, l, r) => write!(fmt, "({} {} {})", op, l, r),
            Self::Unary(op, expr) => write!(fmt, "({}{})", op, expr),
            Self::Paren(expr) => write!(fmt, "({})", expr),
            Self::Assign(l, r) => write!(fmt, "{} = {}", l, r),
            Self::Block(block) => write!(fmt, "{}", block),
            Self::Path(path) => write!(fmt, "{}", path),
            Self::Tuple(xs) => write!(fmt, "({})", util::join(xs, ",")),
            Self::Call(f, args) => write!(fmt, "({} {})", f, util::join(args, " ")),
            Self::Struct(_path, _fields) => todo!(),
            Self::Field(expr, ident) => write!(fmt, "{}.{}", expr, ident),
            Self::While(expr, block) => write!(fmt, "while {} {}", expr, block),
            Self::Loop(block) => write!(fmt, "loop {}", block),
            Self::Closure(name, sig, body) => match name {
                Some(name) => write!(fmt, "fn {} ({}) => {}", name, sig, body),
                None => write!(fmt, "fn ({}) => {}", sig, body),
            },
            Self::Ret(expr) => match expr {
                Some(expr) => write!(fmt, "{}", expr),
                None => write!(fmt, ""),
            },
            Self::If(c, l, r) => match r {
                Some(r) => write!(fmt, "if {} {} {}", c, l, r),
                None => write!(fmt, "if {} {}", c, l),
            },
            Self::Match(_, _) => todo!(),
            Self::Err => write!(fmt, "<expr-err>"),
        }
    }
}
