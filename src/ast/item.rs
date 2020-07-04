use crate::lexer::Span;

#[derive(Debug)]
crate struct Item {
    pub span: Span,
    pub kind: ItemKind,
}

#[derive(Debug)]
crate enum ItemKind {}
