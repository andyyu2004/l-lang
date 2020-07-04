use crate::arena::DroplessArena as Arena;

crate struct LoweringCtx<'ir> {
    crate arena: &'ir Arena,
}

impl<'ir> LoweringCtx<'ir> {
    pub fn new(arena: &'ir Arena) -> Self {
        Self { arena }
    }
}
