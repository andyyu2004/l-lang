use super::Span;

pub struct Item {
    pub span: Span,
    pub kind: ItemKind,
}

pub enum ItemKind {}
