use lc_ast::Ast;

pub struct MacroExpander {}

impl MacroExpander {
    pub fn new() -> Self {
        Self {}
    }

    pub fn expand(self, ast: Ast) -> Ast {
        ast
    }
}
