mod collect;
mod escape;
pub mod inference;
mod item_ctx;
mod relate;
mod tables;
#[cfg(test)]
mod tests;
mod tyctx;
mod writeback;

pub use item_ctx::ItemCtx;
pub use relate::{Relate, TypeRelation};
pub use tables::TypeckOutputs;
pub use tyctx::{GlobalCtx, TyCtx};
