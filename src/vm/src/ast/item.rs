use super::*;
use crate::{lexer::Symbol, span::Span};
use indexed_vec::Idx;

#[derive(Debug, PartialEq, Clone)]
crate struct Item {
    pub span: Span,
    pub id: NodeId,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: ItemKind,
}

#[derive(Debug, PartialEq, Clone)]
crate enum ItemKind {
    Fn(FnSig, Generics, Option<P<Block>>),
}

/// painfully wraps an expression in a function that contains a single expression statement
impl From<Expr> for Item {
    fn from(expr: Expr) -> Self {
        let span = Span { lo: 0, hi: 0 };
        let id = NodeId::new(0);
        let fn_sig = FnSig { inputs: vec![], output: None };
        let generics = Generics { span, id };
        let stmt = Stmt { span, id, kind: StmtKind::Semi(box expr) };
        let block = Some(box Block { span, id, stmts: vec![box stmt] });
        let kind = ItemKind::Fn(fn_sig, generics, block);
        Self {
            span,
            id,
            vis: Visibility { node: VisibilityKind::Private, span },
            ident: Ident { id, span, symbol: Symbol(0) },
            kind,
        }
    }
}
