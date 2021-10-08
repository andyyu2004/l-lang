use super::*;
use lex::TokenTree;
use span::Span;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct Item<K = ItemKind> {
    pub span: Span,
    pub id: NodeId,
    pub vis: Visibility,
    pub ident: Ident,
    pub kind: K,
}

impl Item {
    pub fn generics(&self) -> Option<&Generics> {
        match &self.kind {
            ItemKind::Impl { generics, .. }
            | ItemKind::Fn(_, generics, _)
            | ItemKind::Struct(generics, _)
            | ItemKind::TypeAlias(generics, _)
            | ItemKind::Trait { generics, .. }
            | ItemKind::Enum(generics, _) => Some(generics),
            ItemKind::Macro(..) | ItemKind::Mod(..) | ItemKind::Use(..) | ItemKind::Extern(..) =>
                None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ItemKind {
    /// fn f() {}
    Fn(FnSig, Generics, Option<P<Expr>>),
    /// macro m() { ... }
    Macro(Macro),
    /// enum E {}
    Enum(Generics, Vec<Variant>),
    /// struct S {}
    Struct(Generics, VariantKind),
    /// extern "<abi>" {}
    Extern(Abi, Vec<P<ForeignItem>>),
    /// type T = S;
    TypeAlias(Generics, P<Ty>),
    /// mod foo;
    Mod(Module),
    /// use some::path;
    Use(Path),
    Trait {
        generics: Generics,
        items: Vec<P<TraitItem>>,
    },
    /// impl Trait for Type {}
    /// impl Type {}
    Impl {
        generics: Generics,
        trait_path: Option<Path>,
        self_ty: P<Ty>,
        items: Vec<P<AssocItem>>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Module {
    // no identifier stored here
    // refer to the `Item` struct wrapping this
    pub span: Span,
    pub items: Vec<P<Item>>,
}

impl ItemKind {
    pub fn descr(&self) -> &str {
        match self {
            ItemKind::Fn(_, _, body) => match body {
                Some(_) => "function",
                None => "bodyless function",
            },
            ItemKind::Macro(_) => "macro",
            ItemKind::Enum(..) => "enum",
            ItemKind::Struct(..) => "struct",
            ItemKind::Impl { .. } => "impl block",
            ItemKind::Extern(..) => "extern block",
            ItemKind::TypeAlias(..) => "type alias",
            ItemKind::Use(..) => "use import",
            ItemKind::Mod(..) => "module",
            ItemKind::Trait { .. } => "trait",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssocItemKind {
    Fn(FnSig, Generics, Option<P<Expr>>),
}

impl TryFrom<ItemKind> for AssocItemKind {
    type Error = ItemKind;

    fn try_from(kind: ItemKind) -> Result<Self, Self::Error> {
        match kind {
            ItemKind::Fn(sig, generics, expr) => Ok(Self::Fn(sig, generics, expr)),
            ItemKind::TypeAlias(..) => todo!("assoc types not impl"),
            ItemKind::Use(..)
            | ItemKind::Mod(..)
            | ItemKind::Extern(..)
            | ItemKind::Enum(..)
            | ItemKind::Struct(..)
            | ItemKind::Macro(..)
            | ItemKind::Trait { .. }
            | ItemKind::Impl { .. } => Err(kind),
        }
    }
}

pub type AssocItem = Item<AssocItemKind>;
// we can use identical representation for trait item currently
// as the valid impl items kinds are the same as trait items
// associated types, constants, and functions
pub type TraitItem = AssocItem;
pub type ForeignItem = Item<ForeignItemKind>;

#[derive(Debug, PartialEq, Clone)]
pub enum ForeignItemKind {
    Fn(FnSig, Generics),
}

impl TryFrom<ItemKind> for ForeignItemKind {
    type Error = ItemKind;

    fn try_from(kind: ItemKind) -> Result<Self, Self::Error> {
        match kind {
            ItemKind::Fn(sig, generics, expr) if expr.is_none() => Ok(Self::Fn(sig, generics)),
            _ => Err(kind),
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ItemKind::Fn(sig, _generics, body) => writeln!(
                f,
                "{} fn {}({}) -> {:?} {}",
                self.vis.node,
                self.ident,
                lutil::join(&sig.params, ", "),
                sig.ret_ty,
                body.as_ref().unwrap()
            ),
            ItemKind::Macro(m) => write!(f, "{} macro {}, {}", self.vis.node, self.ident, m),
            ItemKind::TypeAlias(generics, ty) =>
                write!(f, "{} type {}<{}> = {}", self.vis.node, self.ident, generics, ty),
            ItemKind::Enum(_generics, _variants) => todo!(),
            ItemKind::Struct(_generics, _variant_kind) => todo!(),
            ItemKind::Extern(..) => todo!(),
            ItemKind::Use(path) => write!(f, "use {}", path),
            ItemKind::Mod(..) => todo!(),
            ItemKind::Impl { .. } => todo!(),
            ItemKind::Trait { .. } => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Macro {
    pub rules: Vec<MacroRule>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MacroRule {
    pub matcher: MacroMatcher,
    pub transcriber: TokenTree,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FragmentSpecifier {
    Item,
    Block,
    Stmt,
    Pat,
    Expr,
    Ty,
    Ident,
    Path,
    Tt,
    Lit,
    Err,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MacroMatcher {
    pub matches: Vec<MacroMatch>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MacroMatch {
    Token(Token),
    Matcher(Box<MacroMatch>),
    Repetition(Vec<MacroMatch>, Option<Token>, MacroRepetitor),
    Fragment(Ident, FragmentSpecifier),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MacroRepetitor {
    Star,
    Plus,
    Opt,
}

impl Display for Macro {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl Display for MacroRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
